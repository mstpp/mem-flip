use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;

struct App {
    current_screen: usize,
    input: String,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            current_screen: 0,
            input: String::new(),
            should_quit: false,
        }
    }

    fn next_screen(&mut self) {
        self.current_screen = (self.current_screen + 1) % 3; // 3 screens total
        self.input.clear();
    }

    fn get_content(&self) -> Vec<String> {
        match self.current_screen {
            0 => vec![
                "Welcome to Interactive Terminal App!".to_string(),
                "".to_string(),
                "This is screen 1 of 3.".to_string(),
                "Learn how to navigate using the menu below.".to_string(),
            ],
            1 => vec![
                "Screen 2: Input Example".to_string(),
                "".to_string(),
                "Type something and press Enter.".to_string(),
                format!("Your input: {}", self.input),
            ],
            2 => vec![
                "Screen 3: Final Screen".to_string(),
                "".to_string(),
                "Great job! You've navigated through all screens.".to_string(),
                "Press 'q' to quit or 'n' to go back to the beginning.".to_string(),
            ],
            _ => vec!["Error".to_string()],
        }
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key_event(key, app);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key_event(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('n') => app.next_screen(),
        KeyCode::Char(c) if app.current_screen == 1 => app.input.push(c),
        KeyCode::Backspace if app.current_screen == 1 => {
            app.input.pop();
        }
        KeyCode::Enter if app.current_screen == 1 => {
            // Process input here if needed
        }
        _ => {}
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Main content
            Constraint::Length(3), // Menu
        ])
        .split(f.area());

    // Main content
    let content = app.get_content();
    let text: Vec<Line> = content.iter().map(|s| Line::from(s.as_str())).collect();

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Screen {}/3", app.current_screen + 1)),
    );
    f.render_widget(paragraph, chunks[0]);

    // Menu
    let menu_text = vec![Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" = Quit  "),
        Span::styled(
            "n",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" = Next"),
    ])];

    let menu =
        Paragraph::new(menu_text).block(Block::default().borders(Borders::ALL).title("Menu"));
    f.render_widget(menu, chunks[1]);
}
