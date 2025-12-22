use crate::*;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use bitsy_reparser as bs;
use core::cell::OnceCell;
use firefly_rust as ff;

static mut STATE: OnceCell<State> = OnceCell::new();

pub struct State {
    pub game: bs::Game,
    pub room: usize,
    pub frame: u8,
    pub room_dirty: bool,
    pub held_for: u32,
    /// Input on the previous frame.
    pub dpad: ff::DPad,
    /// Currently active dialog.
    pub dialog: Dialog,
    pub script_state: bitsy_script::State,
    /// Tiles in the current room.
    pub tiles: Vec<(u8, bs::Tile)>,
    pub font: ff::FileBuf,
}

impl State {
    pub fn pos(&self) -> bs::Position {
        bs::Position {
            x: self.script_state.pos_x,
            y: self.script_state.pos_y,
        }
    }

    pub fn set_pos(&mut self, pos: bs::Position) {
        self.script_state.pos_x = pos.x;
        self.script_state.pos_y = pos.y;
    }

    pub fn set_room(&mut self, room_id: String) {
        let maybe_room = self.game.rooms.iter().position(|room| room.id == room_id);
        let Some(room_idx) = maybe_room else {
            return;
        };
        self.room = room_idx;
        self.script_state.room = room_id;

        let room = &self.game.rooms[room_idx];
        if let Some(pal) = &room.palette_id {
            self.script_state.palette = pal.clone();
        }
        self.reload_tiles();
        self.room_dirty = true;
    }

    fn reload_tiles(&mut self) {
        let room = &self.game.rooms[self.room];
        self.tiles.clear();
        for (tile_id, i) in room.tiles.iter().zip(0u8..) {
            if tile_id == "0" {
                continue;
            }
            let Some(tile) = &self.game.get_tile(tile_id) else {
                continue;
            };
            let tile = (*tile).clone();
            self.tiles.push((i, tile));
        }
    }
}

fn set_state(state: State) {
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

pub fn load_state() {
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

    let mut script_state = bitsy_script::State::default();
    for var in &game.variables {
        let val = bitsy_script::Val::new(&var.initial_value);
        script_state.vars.set(var.id.to_string(), val);
    }

    let char_width = font.as_font().char_width();
    let char_height = font.as_font().char_height();
    let dialog = Dialog::new(&game.name, &mut script_state, char_width, char_height);
    let state = State {
        game,
        font,
        room: 0,
        frame: 0,
        held_for: 0,
        room_dirty: true,
        dpad: ff::DPad::default(),
        dialog,
        tiles: Vec::new(),
        script_state,
    };
    set_state(state);
    set_starting_room();
}

fn set_starting_room() {
    let state = get_state();
    let Some(avatar) = state.game.get_avatar() else {
        return;
    };
    state.script_state.avatar = avatar.id.clone();
    if let Some(pos) = avatar.position {
        state.script_state.pos_x = pos.x;
        state.script_state.pos_y = pos.y;
    }
    let Some(room_id) = &avatar.room_id else {
        return;
    };
    state.set_room(room_id.clone());
}
