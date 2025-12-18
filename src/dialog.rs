use core::str::Chars;

use crate::*;
use alloc::{string::String, vec, vec::Vec};

pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str) -> Self {
        // Remove triple quotes around the dialog
        const TRIPLE_QUOTE: &str = r#"""""#;
        let mut dialog = dialog;
        if let Some(new_dialog) = dialog.strip_prefix(TRIPLE_QUOTE) {
            dialog = new_dialog.strip_suffix(TRIPLE_QUOTE).unwrap_or(dialog);
        }

        let tokenizer = Tokenizer::new(dialog);
        let mut words: Vec<Word> = Vec::new();
        let mut effect = TextEffect::None;
        for token in tokenizer {
            match token {
                Token::TagBr => {}
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

        let line = Line { words };
        let lines = vec![line];
        let page = Page { lines };
        let pages = vec![page];
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
}

impl<'a> Tokenizer<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            buffer: text.chars(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut word = String::new();
        let mut found_letter = false;
        loop {
            let Some(ch) = self.buffer.next() else {
                break;
            };
            word.push(ch);
            if ch == '\n' {
                return Some(Token::TagBr);
            }
            if ch.is_ascii_whitespace() {
                if found_letter {
                    break;
                }
            } else {
                found_letter = true
            }
        }
        if word.is_empty() {
            return None;
        }
        Some(Token::Word(word))
    }
}
