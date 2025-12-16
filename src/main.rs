#![no_std]
#![no_main]
extern crate alloc;

mod rendering;
mod updating;

use crate::rendering::*;
use crate::updating::*;
use bitsy_nostd_parser as bs;
use core::cell::OnceCell;
use firefly_rust as ff;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    game: bs::Game,
    room: usize,
    pos: bs::Position,
    frame: u8,
    dpad: ff::DPad,
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
    ff::log_debug(&game.name);
    let state = State {
        game,
        room: 0,
        frame: 0,
        pos: bs::Position { x: 0, y: 0 },
        dpad: ff::DPad::default(),
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
    for (i, room) in state.game.rooms.iter().enumerate() {
        if &room.id == room_id {
            state.room = i;
            return;
        }
    }
}
