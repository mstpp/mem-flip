use std::collections::HashMap;
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget, Wrap},
};
use serde::{Deserialize, Serialize};

static CARDS_FILE: &str = "flashcards.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Flashcard {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topics {
    pub topics_map: HashMap<String, Vec<Flashcard>>,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    // Load topics from file, or create empty if file doesn't exist
    let topics = match std::fs::File::open(CARDS_FILE) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            // Return new empty map if file has bad data
            serde_json::from_reader(reader).unwrap_or_else(|_| Topics {
                topics_map: HashMap::new(),
            })
        }
        Err(_) => Topics {
            topics_map: HashMap::new(),
        },
    };

    let mut app = App::new(topics);
    let app_result = app.run(&mut terminal);

    ratatui::restore();

    // Save topics to disk before exiting
    if let Err(e) = app.save_to_disk() {
        eprintln!("Error saving topics: {}", e);
    }

    app_result
}

// Represents different screens in the app
#[derive(Debug, Clone)]
enum AppState {
    TopicSelection,
    FlashcardReview {
        topic: String,
        card_index: usize,
        show_answer: bool,
    },
    CreateTopic {
        input: String,
    },
    AddCard {
        topic: String,
        question_input: String,
        answer_input: String,
        editing_question: bool, // true = editing question, false = editing answer
    },
}

#[derive(Debug)]
pub struct App {
    topics: Topics,
    state: AppState,
    list_state: ListState,
    exit: bool,
}

impl App {
    pub fn new(topics: Topics) -> App {
        let mut list_state = ListState::default();
        // Select first item by default if topics exist
        if !topics.topics_map.is_empty() {
            list_state.select(Some(0));
        }

        App {
            topics,
            state: AppState::TopicSelection,
            list_state,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                self.handle_key_event(key_event);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match &self.state.clone() {
            AppState::TopicSelection => self.handle_topic_selection_keys(key_event),
            AppState::FlashcardReview {
                topic,
                card_index,
                show_answer,
            } => self.handle_flashcard_keys(key_event, topic, *card_index, *show_answer),
            AppState::CreateTopic { input } => self.handle_create_topic_keys(key_event, &input),
            AppState::AddCard {
                topic,
                question_input,
                answer_input,
                editing_question,
            } => self.handle_add_card_keys(
                key_event,
                topic,
                question_input,
                answer_input,
                *editing_question,
            ),
        }
    }

    fn handle_topic_selection_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('n') => {
                self.state = AppState::CreateTopic {
                    input: String::new(),
                };
            }
            KeyCode::Char('a') => {
                // Add card to selected topic
                if let Some(selected) = self.list_state.selected() {
                    let topic_name = self.get_sorted_topics()[selected].clone();
                    self.state = AppState::AddCard {
                        topic: topic_name,
                        question_input: String::new(),
                        answer_input: String::new(),
                        editing_question: true,
                    };
                }
            }
            KeyCode::Enter => {
                // Enter topic for flashcard review
                if let Some(selected) = self.list_state.selected() {
                    let topic_name = self.get_sorted_topics()[selected].clone();

                    // Only enter if topic has cards
                    if let Some(cards) = self.topics.topics_map.get(&topic_name) {
                        if !cards.is_empty() {
                            self.state = AppState::FlashcardReview {
                                topic: topic_name,
                                card_index: 0,
                                show_answer: false,
                            };
                        }
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.select_next_topic(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_topic(),
            _ => {}
        }
    }

    fn handle_flashcard_keys(
        &mut self,
        key_event: KeyEvent,
        topic: &str,
        card_index: usize,
        show_answer: bool,
    ) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::TopicSelection;
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                // Toggle answer visibility
                self.state = AppState::FlashcardReview {
                    topic: topic.to_string(),
                    card_index,
                    show_answer: !show_answer,
                };
            }
            KeyCode::Char('n') | KeyCode::Right => {
                // Next card
                if let Some(cards) = self.topics.topics_map.get(topic) {
                    let next_index = (card_index + 1) % cards.len();
                    self.state = AppState::FlashcardReview {
                        topic: topic.to_string(),
                        card_index: next_index,
                        show_answer: false,
                    };
                }
            }
            KeyCode::Char('p') | KeyCode::Left => {
                // Previous card
                if let Some(cards) = self.topics.topics_map.get(topic) {
                    let prev_index = if card_index == 0 {
                        cards.len() - 1
                    } else {
                        card_index - 1
                    };
                    self.state = AppState::FlashcardReview {
                        topic: topic.to_string(),
                        card_index: prev_index,
                        show_answer: false,
                    };
                }
            }
            _ => {}
        }
    }

    fn handle_create_topic_keys(&mut self, key_event: KeyEvent, current_input: &str) {
        let mut input = current_input.to_string();

        match key_event.code {
            KeyCode::Esc => {
                self.state = AppState::TopicSelection;
            }
            KeyCode::Enter => {
                if !input.trim().is_empty() {
                    // Create new topic
                    self.topics
                        .topics_map
                        .insert(input.trim().to_string(), Vec::new());
                    self.state = AppState::TopicSelection;
                    // Select the newly created topic
                    self.update_list_selection();
                }
            }
            KeyCode::Char(c) => {
                input.push(c);
                self.state = AppState::CreateTopic { input };
            }
            KeyCode::Backspace => {
                input.pop();
                self.state = AppState::CreateTopic { input };
            }
            _ => {}
        }
    }

    fn handle_add_card_keys(
        &mut self,
        key_event: KeyEvent,
        topic: &str,
        question: &str,
        answer: &str,
        editing_question: bool,
    ) {
        match key_event.code {
            KeyCode::Esc => {
                self.state = AppState::TopicSelection;
            }

            KeyCode::Tab => {
                // Switch between question and answer input
                self.state = AppState::AddCard {
                    topic: topic.to_string(),
                    question_input: question.to_string(),
                    answer_input: answer.to_string(),
                    editing_question: !editing_question,
                };
            }

            // KeyCode::Enter
            // // this is on macos: SHIFT+OPTION+ENTER
            //     if key_event
            //         .modifiers
            //         .contains(crossterm::event::KeyModifiers::ALT) =>
            KeyCode::Enter => {
                // Plain Enter: Add newline
                if editing_question {
                    let mut q = question.to_string();
                    q.push('\n');
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: q,
                        answer_input: answer.to_string(),
                        editing_question,
                    };
                } else {
                    let mut a = answer.to_string();
                    a.push('\n');
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: question.to_string(),
                        answer_input: a,
                        editing_question,
                    };
                }
            }

            KeyCode::Char('s')
            // CONTROL + S on macos
                if key_event
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER) =>
            {
                // Ctrl+S or Cmd+S: Save card
                if !question.trim().is_empty() && !answer.trim().is_empty() {
                    let flashcard = Flashcard {
                        question: question.trim().to_string(),
                        answer: answer.trim().to_string(),
                    };

                    if let Some(cards) = self.topics.topics_map.get_mut(topic) {
                        cards.push(flashcard);
                    }

                    self.state = AppState::TopicSelection;
                }
            }

            KeyCode::Char(c) => {
                if editing_question {
                    let mut q = question.to_string();
                    q.push(c);
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: q,
                        answer_input: answer.to_string(),
                        editing_question,
                    };
                } else {
                    let mut a = answer.to_string();
                    a.push(c);
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: question.to_string(),
                        answer_input: a,
                        editing_question,
                    };
                }
            }

            KeyCode::Backspace => {
                if editing_question {
                    let mut q = question.to_string();
                    q.pop();
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: q,
                        answer_input: answer.to_string(),
                        editing_question,
                    };
                } else {
                    let mut a = answer.to_string();
                    a.pop();
                    self.state = AppState::AddCard {
                        topic: topic.to_string(),
                        question_input: question.to_string(),
                        answer_input: a,
                        editing_question,
                    };
                }
            }
            _ => {}
        }
    }

    fn select_next_topic(&mut self) {
        let topics_count = self.topics.topics_map.len();
        if topics_count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % topics_count,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous_topic(&mut self) {
        let topics_count = self.topics.topics_map.len();
        if topics_count == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    topics_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn update_list_selection(&mut self) {
        let topics_count = self.topics.topics_map.len();
        if topics_count > 0 {
            self.list_state.select(Some(0));
        }
    }

    fn get_sorted_topics(&self) -> Vec<String> {
        let mut topics: Vec<_> = self.topics.topics_map.keys().cloned().collect();
        topics.sort();
        topics
    }

    fn save_to_disk(&self) -> io::Result<()> {
        let file = std::fs::File::create(CARDS_FILE)?;
        serde_json::to_writer_pretty(file, &self.topics)?;
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.state {
            AppState::TopicSelection => self.render_topic_selection(area, buf),
            AppState::FlashcardReview {
                topic,
                card_index,
                show_answer,
            } => self.render_flashcard(area, buf, topic, *card_index, *show_answer),
            AppState::CreateTopic { input } => self.render_create_topic(area, buf, input),
            AppState::AddCard {
                topic,
                question_input,
                answer_input,
                editing_question,
            } => self.render_add_card(
                area,
                buf,
                topic,
                question_input,
                answer_input,
                *editing_question,
            ),
        }
    }
}

// Separate rendering logic for each state
impl App {
    fn render_topic_selection(&self, area: Rect, buf: &mut Buffer) {
        let title = " 💾 Memory Flip Flashcards ";
        let instructions = vec![
            " Navigate ".into(),
            "<↑↓>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " New Topic ".into(),
            "<N>".blue().bold(),
            " Add Card ".into(),
            "<A>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ];

        let topics = self.get_sorted_topics();

        if topics.is_empty() {
            // Show empty state
            let empty_text = "No topics yet!\n\nPress 'N' to create your first topic.";
            Paragraph::new(empty_text)
                .left_aligned()
                .block(
                    Block::bordered()
                        .title(title.bold().into_left_aligned_line())
                        .title_bottom(Line::from(instructions).left_aligned()),
                )
                .render(area, buf);
            return;
        }

        // Create list items
        let items: Vec<ListItem> = topics
            .iter()
            .map(|topic| {
                let card_count = self
                    .topics
                    .topics_map
                    .get(topic)
                    .map(|cards| cards.len())
                    .unwrap_or(0);

                let content = format!("  {}  ({} cards)", topic, card_count);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(title.bold().into_left_aligned_line())
                    .title_bottom(Line::from(instructions).left_aligned()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        // Use StatefulWidget for list with selection
        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.list_state.clone());
    }

    fn render_flashcard(
        &self,
        area: Rect,
        buf: &mut Buffer,
        topic: &str,
        card_index: usize,
        show_answer: bool,
    ) {
        let instructions = vec![
            " Flip ".into(),
            "<Space>".blue().bold(),
            " Previous ".into(),
            "<P/←>".blue().bold(),
            " Next ".into(),
            "<N/→>".blue().bold(),
            " Back ".into(),
            "<Esc> ".blue().bold(),
        ];

        if let Some(cards) = self.topics.topics_map.get(topic) {
            if let Some(card) = cards.get(card_index) {
                let progress = format!(" Card {}/{} ", card_index + 1, cards.len());

                // Split area into two sections
                let chunks =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(area);

                // Render question (top half)
                let question_text = format!("Q: {}", card.question);
                Paragraph::new(question_text)
                    .wrap(Wrap { trim: true })
                    .left_aligned()
                    .block(
                        Block::bordered()
                            .title(
                                format!(" 📝 {} {} ", topic, progress)
                                    .bold()
                                    .into_left_aligned_line(),
                            )
                            .style(Style::default().fg(Color::Cyan)),
                    )
                    .render(chunks[0], buf);

                // Render answer (bottom half) - only if show_answer is true
                let answer_content = if show_answer {
                    format!("A: {}", card.answer)
                } else {
                    "[Press Space to reveal answer]".to_string()
                };

                let answer_style = if show_answer {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                Paragraph::new(answer_content)
                    .wrap(Wrap { trim: true })
                    .left_aligned()
                    .block(
                        Block::bordered()
                            .title_bottom(Line::from(instructions).left_aligned())
                            .style(answer_style),
                    )
                    .render(chunks[1], buf);

                return;
            }
        }

        // Fallback if no card found
        Paragraph::new("No cards available")
            .left_aligned()
            .block(Block::bordered())
            .render(area, buf);
    }

    fn render_create_topic(&self, area: Rect, buf: &mut Buffer, input: &str) {
        let text = vec![
            Line::from(""),
            Line::from("Enter topic name:"),
            Line::from(""),
            Line::from(vec![
                Span::raw("> "),
                Span::styled(input, Style::default().fg(Color::Yellow)), // Use input directly
                Span::styled("█", Style::default().fg(Color::Yellow)),
            ]),
        ];

        let instructions = " Press Enter to create | Esc to cancel ";

        Paragraph::new(text)
            .left_aligned()
            .block(
                Block::bordered()
                    .title(" ➕ New Topic ".bold().into_left_aligned_line())
                    .title_bottom(instructions),
            )
            .render(area, buf);
    }

    fn render_add_card(
        &self,
        area: Rect,
        buf: &mut Buffer,
        topic: &str,
        question: &str,
        answer: &str,
        editing_question: bool,
    ) {
        let chunks = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ])
        .split(area);

        // Question input
        let question_style = if editing_question {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // let question_text = if question.is_empty() && editing_question {
        //     vec![
        //         Line::from(""),
        //         Line::from(vec![Span::raw("> "), Span::styled("█", question_style)]),
        //     ]
        // } else {
        //     vec![
        //         Line::from(""),
        //         Line::from(vec![
        //             Span::raw("> "),
        //             Span::styled(question, question_style),
        //             if editing_question {
        //                 Span::styled("█", question_style)
        //             } else {
        //                 Span::raw("")
        //             },
        //         ]),
        //     ]
        // };

        let question_text = if question.is_empty() && editing_question {
            vec![
                Line::from(""),
                Line::from(vec![Span::raw("> "), Span::styled("█", question_style)]),
            ]
        } else {
            let question_lines: Vec<&str> = question.split('\n').collect();
            let num_lines = question_lines.len();

            std::iter::once(Line::from("")) // Empty line at top
                .chain(question_lines.iter().enumerate().map(|(i, line)| {
                    let mut spans = vec![Span::raw("> "), Span::styled(*line, question_style)];

                    // Cursor on last line when editing
                    if editing_question && i == num_lines - 1 {
                        spans.push(Span::styled("█", question_style));
                    }

                    Line::from(spans)
                }))
                .collect()
        };

        Paragraph::new(question_text)
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(format!(
                        " Question {} ",
                        if editing_question { "✎" } else { "" }
                    ))
                    .style(if editing_question {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .render(chunks[0], buf);

        // Answer input
        let answer_style = if !editing_question {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // let answer_text = if answer.is_empty() && !editing_question {
        //     vec![
        //         Line::from(""),
        //         Line::from(vec![Span::raw("> "), Span::styled("█", answer_style)]),
        //     ]
        // } else {
        //     vec![
        //         Line::from(""),
        //         Line::from(vec![
        //             Span::raw("> "),
        //             Span::styled(answer, answer_style),
        //             if !editing_question {
        //                 Span::styled("█", answer_style)
        //             } else {
        //                 Span::raw("")
        //             },
        //         ]),
        //     ]
        // };

        let answer_text = if answer.is_empty() && !editing_question {
            vec![
                Line::from(""),
                Line::from(vec![Span::raw("> "), Span::styled("█", answer_style)]),
            ]
        } else {
            let answer_lines: Vec<&str> = answer.split('\n').collect();
            let num_lines = answer_lines.len();

            std::iter::once(Line::from("")) // Empty line at top
                .chain(answer_lines.iter().enumerate().map(|(i, line)| {
                    let mut spans = vec![Span::raw("> "), Span::styled(*line, answer_style)];

                    // Cursor on last line when editing answer
                    if !editing_question && i == num_lines - 1 {
                        spans.push(Span::styled("█", answer_style));
                    }

                    Line::from(spans)
                }))
                .collect()
        };

        Paragraph::new(answer_text)
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(format!(
                        " Answer {} ",
                        if !editing_question { "✎" } else { "" }
                    ))
                    .style(if !editing_question {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .render(chunks[1], buf);

        // Instructions
        let instructions = vec![
            Line::from(""),
            Line::from(vec![
                " Switch field ".into(),
                "<Tab>".blue().bold(),
                " Save ".into(),
                // "<Shift + Opt + Enter>".green().bold(),
                "<CTL + S >".green().bold(),
                " Cancel ".into(),
                "<Esc> ".red().bold(),
            ]),
        ];

        Paragraph::new(instructions)
            .left_aligned()
            .block(Block::bordered().title(format!(" 📝 Add Card to '{}' topic", topic)))
            .render(chunks[2], buf);
    }
}
