#![no_std]
#![no_main]
extern crate alloc;

mod rendering;

use crate::rendering::*;
use bitsy_nostd_parser as bs;
use core::cell::OnceCell;
use firefly_rust as ff;

static mut STATE: OnceCell<State> = OnceCell::new();

struct State {
    game: bs::Game,
    room: usize,
    frame: u8,
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
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
    set_starting_room();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.frame = (state.frame + 1) % 60;
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    render_room(state);
}

fn set_starting_room() {
    let state = get_state();
    let Ok(avatar) = state.game.get_avatar() else {
        return;
    };
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
