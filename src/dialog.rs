use crate::*;
use alloc::vec::Vec;
use bitsy_script as bs;

#[derive(Default)]
pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str, state: &mut bs::State, char_width: u8) -> Self {
        const TRIPLE_QUOTE: &str = r#"""""#;
        const LINES_PER_PAGE: usize = 2;
        const LINE_WIDTH: usize = firefly_rust::WIDTH as usize;

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
        let mut line_width = 0;
        for word in interpreter {
            use bitsy_script::Word::*;
            match word {
                LineBreak => {
                    if !words.is_empty() {
                        lines.push(Line { words });
                        words = Vec::new();
                        line_width = 0;
                    }
                    if lines.len() >= LINES_PER_PAGE {
                        pages.push(Page { lines });
                        lines = Vec::new();
                    }
                }
                PageBreak => {
                    if !words.is_empty() {
                        lines.push(Line { words });
                        words = Vec::new();
                        line_width = 0;
                    }
                    if !lines.is_empty() {
                        pages.push(Page { lines });
                        lines = Vec::new();
                    }
                }
                w => {
                    let n_chars: usize = if let Text(t, _) = &w { t.len() } else { 8 };
                    let word_width = n_chars * usize::from(char_width);
                    if line_width + word_width > LINE_WIDTH {
                        if !words.is_empty() {
                            lines.push(Line { words });
                            words = Vec::new();
                            line_width = 0;
                        }
                        if lines.len() >= LINES_PER_PAGE {
                            pages.push(Page { lines });
                            lines = Vec::new();
                        }
                    }
                    line_width += word_width;
                    words.push(w)
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
    pub words: Vec<bs::Word>,
}
