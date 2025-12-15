#![no_std]
#![no_main]
extern crate alloc;
use alloc::borrow::ToOwned;
use bitsy_nostd_parser::*;
use firefly_rust::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    let raw = load_file_buf("main").unwrap();
    let raw = alloc::str::from_utf8(raw.data()).unwrap();
    let (game, _) = Game::from(raw.to_owned()).unwrap();
    log_debug(&game.name);
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    // ...
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    clear_screen(Color::White);
}
