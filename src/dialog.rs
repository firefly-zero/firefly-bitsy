use crate::*;
use alloc::vec::Vec;
use bitsy_script as bs;
use firefly_rust as ff;

#[derive(Default)]
pub struct Dialog {
    pub pages: Vec<Page>,
}

impl Dialog {
    pub fn new(dialog: &str, state: &mut bs::State, char_width: u8, char_height: u8) -> Self {
        let builder = DialogBuilder {
            char_width,
            char_height,
            ..Default::default()
        };
        builder.build(dialog, state)
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
    pub words: Vec<Word>,
    /// If the renderer started to render the page on the screen.
    ///
    /// When false, the renderer will first clear the region to hide the old page.
    pub started: bool,
    /// If true, stop the words animation and render the whole page in one go.
    pub fast: bool,
}

impl Page {
    pub fn all_rendered(&self) -> bool {
        for word in &self.words {
            if !word.rendered {
                return false;
            }
        }
        true
    }
}

pub struct Word {
    pub word: bs::Word,
    pub point: ff::Point,
    pub rendered: bool,
}

#[derive(Default)]
struct DialogBuilder {
    pages: Vec<Page>,
    words: Vec<Word>,
    char_width: u8,
    char_height: u8,
    offset_x: usize,
    offset_y: usize,
}

impl DialogBuilder {
    pub fn build(mut self, dialog: &str, state: &mut bs::State) -> Dialog {
        const TRIPLE_QUOTE: &str = r#"""""#;
        const BOX_WIDTH: usize = firefly_rust::WIDTH as usize;
        // const BOX_HEIGHT: usize = firefly_rust::HEIGHT as usize - 128;
        const BOX_HEIGHT: usize = 10;

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
                    if self.offset_y > BOX_HEIGHT {
                        self = self.flush_page();
                    }
                }
                PageBreak => {
                    self = self.flush_line();
                    if !self.words.is_empty() {
                        self = self.flush_page();
                    }
                }
                w => {
                    let n_chars: usize = if let Text(t, _) = &w { t.len() } else { 8 };
                    let word_width = n_chars * usize::from(self.char_width);
                    if self.offset_x + word_width > BOX_WIDTH {
                        self = self.flush_line();
                        if self.offset_y >= BOX_HEIGHT {
                            self = self.flush_page();
                        }
                    }
                    let point = ff::Point::new(self.offset_x as i32, self.offset_y as i32);
                    self.words.push(Word {
                        word: w,
                        point,
                        rendered: false,
                    });
                    self.offset_x += word_width;
                }
            }
        }

        if !self.words.is_empty() {
            self.pages.push(Page {
                words: self.words,
                started: false,
                fast: false,
            });
        }
        Dialog { pages: self.pages }
    }

    fn flush_line(mut self) -> Self {
        if self.offset_x != 0 {
            self.offset_x = 0;
            self.offset_y += usize::from(self.char_height);
        }
        self
    }

    fn flush_page(mut self) -> Self {
        self.offset_x = 0;
        self.offset_y = 0;
        self.pages.push(Page {
            words: self.words,
            started: false,
            fast: false,
        });
        self.words = Vec::new();
        self
    }
}
