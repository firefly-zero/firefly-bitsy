use crate::*;
use alloc::vec::Vec;
use bitsy_script as bs;

#[derive(Default)]
pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str, state: &mut bs::State) -> Self {
        const TRIPLE_QUOTE: &str = r#"""""#;
        const LINES_PER_PAGE: usize = 2;

        // Remove triple quotes around the dialog
        let mut dialog = dialog;
        if let Some(new_dialog) = dialog.strip_prefix(TRIPLE_QUOTE) {
            dialog = new_dialog.strip_suffix(TRIPLE_QUOTE).unwrap_or(dialog);
        }

        let tokenizer = bs::Tokenizer::new(dialog);
        let interpreter = bs::Interpreter {
            tokens: tokenizer,
            state,
        };

        let mut pages = Vec::new();
        let mut lines = Vec::new();
        let mut words: Vec<bs::Word> = Vec::new();
        for word in interpreter {
            use bitsy_script::Word::*;
            match word {
                LineBreak => {
                    if lines.len() >= LINES_PER_PAGE {
                        pages.push(Page { lines });
                        lines = Vec::new();
                    }
                    lines.push(Line { words });
                    words = Vec::new();
                }
                PageBreak => {
                    if !words.is_empty() {
                        lines.push(Line { words });
                        words = Vec::new();
                    }
                    if !lines.is_empty() {
                        pages.push(Page { lines });
                        lines = Vec::new();
                    }
                }
                w => words.push(w),
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
    pub words: Vec<bs::Word>,
}
