use crate::*;
use alloc::vec::Vec;
use bitsy_script as bs;
use firefly_rust as ff;

#[derive(Default)]
pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str, state: &mut bs::State, char_width: u8) -> Self {
        let builder = DialogBuilder::default();
        builder.build(dialog, state, char_width)
    }

    pub fn n_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn current_page(&mut self) -> Option<&mut Page> {
        self.pages.first_mut()
    }

    pub fn next_page(&mut self) {
        let Some(page) = self.pages.first_mut() else {
            return;
        };
        if !page.fast && !page.all_rendered() {
            page.fast = true;
        } else {
            self.pages.remove(0);
        }
    }
}

pub struct Page {
    pub lines: Vec<Line>,
    /// If the renderer started to render the page on the screen.
    ///
    /// When false, the renderer will first clear the region to hide the old page.
    pub started: bool,
    /// If true, stop the words animation and render the whole page in one go.
    pub fast: bool,
}

impl Page {
    pub fn all_rendered(&self) -> bool {
        for line in &self.lines {
            for word in &line.words {
                if !word.rendered {
                    return false;
                }
            }
        }
        true
    }
}

pub struct Line {
    pub words: Vec<Word>,
}

pub struct Word {
    pub word: bs::Word,
    pub point: ff::Point,
    pub rendered: bool,
}

#[derive(Default)]
struct DialogBuilder {
    pages: Vec<Page>,
    lines: Vec<Line>,
    words: Vec<Word>,
    line_width: usize,
}

impl DialogBuilder {
    pub fn build(mut self, dialog: &str, state: &mut bs::State, char_width: u8) -> Dialog {
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

        for word in interpreter {
            use bs::Word::*;
            match word {
                LineBreak => {
                    self = self.flush_line();
                    if self.lines.len() >= LINES_PER_PAGE {
                        self = self.flush_page();
                    }
                }
                PageBreak => {
                    self = self.flush_line();
                    if !self.lines.is_empty() {
                        self = self.flush_page();
                    }
                }
                w => {
                    let n_chars: usize = if let Text(t, _) = &w { t.len() } else { 8 };
                    let word_width = n_chars * usize::from(char_width);
                    if self.line_width + word_width > LINE_WIDTH {
                        self = self.flush_line();
                        if self.lines.len() >= LINES_PER_PAGE {
                            self = self.flush_page();
                        }
                    }
                    self.words.push(Word {
                        word: w,
                        point: ff::Point::new(self.line_width as i32, 0),
                        rendered: false,
                    });
                    self.line_width += word_width;
                }
            }
        }

        if !self.words.is_empty() {
            self.lines.push(Line { words: self.words });
        }
        if !self.lines.is_empty() {
            self.pages.push(Page {
                lines: self.lines,
                started: false,
                fast: false,
            });
        }
        Dialog { pages: self.pages }
    }

    fn flush_line(mut self) -> Self {
        if !self.words.is_empty() {
            self.lines.push(Line { words: self.words });
            self.words = Vec::new();
            self.line_width = 0;
        }
        self
    }

    fn flush_page(mut self) -> Self {
        self.pages.push(Page {
            lines: self.lines,
            started: false,
            fast: false,
        });
        self.lines = Vec::new();
        self
    }
}
