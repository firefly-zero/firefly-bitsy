#![no_std]
#![no_main]
extern crate alloc;

mod rendering;
mod updating;

use crate::rendering::*;
use crate::updating::*;
use alloc::string::String;
use alloc::vec::Vec;
use bitsy_nostd_parser as bs;
use core::cell::OnceCell;
use firefly_rust as ff;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    game: bs::Game,
    room: usize,
    pos: bs::Position,
    frame: u8,
    held_for: u32,
    /// Input on the previous frame.
    dpad: ff::DPad,
    /// Currently active dialog.
    dialog: Vec<String>,
    /// Tiles in the current room.
    tiles: Vec<(u8, bs::Tile)>,
    font: ff::FileBuf,
}

fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

#[unsafe(no_mangle)]
extern "C" fn boot() {
    let raw = ff::load_file_buf("main").unwrap();
    let raw = alloc::str::from_utf8(raw.data()).unwrap();
    let (game, warnings) = match bs::Game::from(raw) {
        Ok(v) => v,
        Err(err) => panic!("{err}"),
    };
    for warning in warnings {
        ff::log_error(warning.as_str());
    }
    let Some(font) = ff::load_file_buf("font") else {
        panic!("font not found")
    };
    let dialog = split_lines(&game.name);
    let state = State {
        game,
        font,
        room: 0,
        frame: 0,
        held_for: 0,
        pos: bs::Position { x: 0, y: 0 },
        dpad: ff::DPad::default(),
        dialog,
        tiles: Vec::new(),
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
    set_starting_room();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    update_state(state);
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    render_room(state);
}

fn set_starting_room() {
    let state = get_state();
    let Some(avatar) = state.game.get_avatar() else {
        return;
    };
    if let Some(pos) = avatar.position {
        state.pos = pos;
    }
    let Some(room_id) = &avatar.room_id else {
        return;
    };
    set_room(state, &room_id.clone());
}
