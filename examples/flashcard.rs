use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Flashcard {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topics {
    pub topics_map: HashMap<String, Vec<Flashcard>>,
}

fn main() {
    let s = r#"{"question": "name?", "answer":"rame"}"#;
    let f: Flashcard = serde_json::from_str(s).unwrap();
    println!("{f:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn test_deser() {
        let s = r#"{"question": "name?", "answer":"rame"}"#;
        let f: Flashcard = serde_json::from_str(s).unwrap();
        assert_str_eq!(f.question.as_str(), "name?");
        assert_str_eq!(f.answer.as_str(), "rame");
    }

    #[test]
    fn test_serial() {
        let f = Flashcard {
            question: "don?".to_string(),
            answer: "corleone".to_string(),
        };
        let ser = serde_json::to_string(&f).unwrap();
        let expected = r#"{"question":"don?","answer":"corleone"}"#;
        assert_str_eq!(expected, ser.as_str());
    }

    #[test]
    fn test_serial_pretty() {
        let f = Flashcard {
            question: "who?".to_string(),
            answer: "me".to_string(),
        };
        let ser = serde_json::to_string_pretty(&f).unwrap();
        let expected = r#"{
  "question": "who?",
  "answer": "me"
}"#;
        assert_str_eq!(expected, ser.as_str());
    }

    // Topics
    //
    #[test]
    fn test_topic_deser() {
        let in_data = r#"{
    "topics_map": {
        "trivia": [
            {
                "question": "most beatuful city?",
                "answer": "Rio"
            }
        ],
        "math": [
            {
                "question": "2+2?",
                "answer": "4"
            },
            {
                "question": "is 7 a prime number?",
                "answer": "yes"
            }
        ]
    }
}"#;
        let res: Result<Topics, _> = serde_json::from_str(in_data);
        println!("{res:?}");
        assert!(res.is_ok());
        // insert val
        let trivia_q2 = Flashcard {
            question: "days around the Sun?".to_string(),
            answer: "365".to_string(),
        };
        let mut topics = res.unwrap();
        topics
            .topics_map
            .entry("trivia".to_string())
            .and_modify(|v| v.push(trivia_q2));

        let deser_pretty = serde_json::to_string_pretty(&topics).unwrap();
        println!("{deser_pretty}");

        let trivia_cards = topics.topics_map.get("trivia").unwrap();
        assert_eq!(trivia_cards.len(), 2);
    }
}
