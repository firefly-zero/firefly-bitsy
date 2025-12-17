pub fn split_lines(dialog: &str) -> Vec<String> {
    // Remove triple quotes around the dialog
    const TRIPLE_QUOTE: &str = r#"""""#;
    let mut dialog = dialog;
    if let Some(new_dialog) = dialog.strip_prefix(TRIPLE_QUOTE) {
        dialog = new_dialog.strip_suffix(TRIPLE_QUOTE).unwrap_or(dialog);
    }

    let mut lines = Vec::new();
    let mut line = String::new();
    const MARGIN_X: i32 = 2;
    const FONT_WIDTH: i32 = 6;
    for word in dialog.split_ascii_whitespace() {
        let n_chars = (word.len() + line.len() + 1) as i32;
        if n_chars * FONT_WIDTH > ff::WIDTH - MARGIN_X * 2 {
            lines.push(line.clone());
            line.clear();
        }
        line.push(' ');
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
