use core::str::Chars;

use crate::*;
use alloc::{string::String, vec, vec::Vec};

pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str) -> Self {
        const TRIPLE_QUOTE: &str = r#"""""#;
        const LINES_PER_PAGE: usize = 2;

        // Remove triple quotes around the dialog
        let mut dialog = dialog;
        if let Some(new_dialog) = dialog.strip_prefix(TRIPLE_QUOTE) {
            dialog = new_dialog.strip_suffix(TRIPLE_QUOTE).unwrap_or(dialog);
        }

        let tokenizer = Tokenizer::new(dialog);
        let mut pages = Vec::new();
        let mut lines = Vec::new();
        let mut words: Vec<Word> = Vec::new();
        let mut effect = TextEffect::None;
        for token in tokenizer {
            match token {
                Token::TagBr => {
                    if lines.len() >= LINES_PER_PAGE {
                        pages.push(Page { lines });
                        lines = Vec::new();
                    }
                    lines.push(Line { words });
                    words = Vec::new();
                }
                Token::TagPg => {}
                Token::TagEff(e) => effect = e,
                Token::TagUnknown => {}
                Token::CloseTag => effect = TextEffect::None,
                Token::Word(text) => {
                    let word = Word { text, effect };
                    words.push(word)
                }
            }
        }

        if !words.is_empty() {
            lines.push(Line { words });
        }
        if !lines.is_empty() {
            pages.push(Page { lines });
        }

        Self { pages }
    }

    pub fn n_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn current_page(&self) -> Option<&Page> {
        self.pages.first()
    }

    pub fn next_page(&mut self) {
        if !self.pages.is_empty() {
            self.pages.remove(0);
        }
    }
}

pub struct Page {
    pub lines: Vec<Line>,
}

pub struct Line {
    pub words: Vec<Word>,
}

pub struct Word {
    pub text: String,
    pub effect: TextEffect,
}

#[derive(Copy, Clone)]
pub enum TextEffect {
    /// No effects.
    None,
    /// {wvy} text in tags waves up and down.
    Wavy,
    /// {shk} text in tags shakes constantly.
    Shaky,
    /// {rbw} text in tags is rainbow colored.
    Rainbow,
    /// {clr} use a palette color for dialog text.
    Color(u8),
}

enum Token {
    /// Line break tag.
    TagBr,
    /// Page break tag.
    TagPg,
    /// Text effect tag.
    TagEff(TextEffect),
    /// Unsupported tag.
    TagUnknown,
    /// A closing tag.
    CloseTag,
    /// A plaintext word.
    Word(String),
}

struct Tokenizer<'a> {
    buffer: Chars<'a>,
    stash: Option<char>,
}

impl<'a> Tokenizer<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            buffer: text.chars(),
            stash: None,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut word = String::new();
        let mut found_letter = false;
        let mut inside_tag = false;
        loop {
            let ch = if let Some(stash) = self.stash.take() {
                stash
            } else if let Some(ch) = self.buffer.next() {
                ch
            } else {
                break;
            };
            word.push(ch);
            match ch {
                '\n' => return Some(Token::TagBr),
                '{' => {
                    if found_letter {
                        self.stash = Some('{');
                        word.pop();
                        break;
                    }
                    inside_tag = true;
                }
                '}' => {
                    if inside_tag {
                        if word.starts_with("{/") {
                            return Some(Token::CloseTag);
                        }
                        let token = match word.as_str() {
                            "{br}" => Token::TagBr,
                            "{pg}" => Token::TagPg,
                            "{clr1}" => Token::TagEff(TextEffect::Color(1)),
                            "{clr2}" => Token::TagEff(TextEffect::Color(2)),
                            "{clr3}" => Token::TagEff(TextEffect::Color(3)),
                            "{clr 1}" => Token::TagEff(TextEffect::Color(1)),
                            "{clr 2}" => Token::TagEff(TextEffect::Color(2)),
                            "{clr 3}" => Token::TagEff(TextEffect::Color(3)),
                            "{wvy}" => Token::TagEff(TextEffect::Wavy),
                            "{shk}" => Token::TagEff(TextEffect::Shaky),
                            "{rbw}" => Token::TagEff(TextEffect::Rainbow),
                            _ => Token::TagUnknown,
                        };
                        return Some(token);
                    }
                    found_letter = true
                }
                '\t' | '\x0C' | '\r' | ' ' => {
                    if !inside_tag && found_letter {
                        break;
                    }
                }
                _ => found_letter = true,
            }
        }
        if word.is_empty() {
            return None;
        }
        Some(Token::Word(word))
    }
}
