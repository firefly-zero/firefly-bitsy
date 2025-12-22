use crate::*;
use bitsy_reparser as bs;
use firefly_rust as ff;

/// The number of tiles in a row.
const TILES_X: u8 = 16;
/// The number of tiles in a column.
const TILES_Y: u8 = 16;

pub fn update_state(state: &mut State) {
    state.frame = (state.frame + 1) % 60;
    handle_pad(state);
    get_avatar(state).position = Some(state.pos());
}

fn handle_pad(state: &mut State) {
    let dpad = read_dpad();
    if dpad.any() {
        state.held_for += 1;
    } else {
        state.held_for = 0;
    }
    let mut old_dpad = state.dpad;
    if state.held_for > 14 && state.held_for.is_multiple_of(4) {
        old_dpad = ff::DPad::default();
    }
    let pressed = dpad.just_pressed(&old_dpad);
    state.dpad = dpad;

    if state.dialog.n_pages() != 0 {
        if pressed.any() {
            state.dialog.next_page();
        }
        return;
    }

    if pressed.left {
        move_avatar_to(state, -1, 0);
    } else if pressed.right {
        move_avatar_to(state, 1, 0);
    } else if pressed.up {
        move_avatar_to(state, 0, -1);
    } else if pressed.down {
        move_avatar_to(state, 0, 1);
    }
}

fn read_dpad() -> firefly_rust::DPad {
    let mut dpad = match ff::read_pad(ff::Peer::COMBINED) {
        Some(pad) => pad.as_dpad(),
        None => ff::DPad::default(),
    };
    let buttons = ff::read_buttons(ff::Peer::COMBINED);
    if buttons.s {
        dpad.down = true;
    }
    if buttons.e {
        dpad.right = true;
    }
    if buttons.w {
        dpad.left = true;
    }
    if buttons.n {
        dpad.up = true;
    }
    dpad
}

fn move_avatar_to(state: &mut State, dx: i8, dy: i8) {
    let old_pos = state.pos();
    let x = old_pos.x.saturating_add_signed(dx).min(TILES_X - 1);
    let y = old_pos.y.saturating_add_signed(dy).min(TILES_Y - 1);
    let new_pos = bs::Position { x, y };

    if let Some(item) = pop_item_at(state, new_pos) {
        let dialog_id = match &item.dialogue_id {
            Some(id) => id,
            None => &item.id,
        };
        let dialog_id = dialog_id.clone();
        show_dialog(state, &dialog_id)
    }

    let left_room = leave_room(state, new_pos);
    if left_room {
        return;
    }

    if let Some(sprite) = get_sprite_at(state, new_pos) {
        let sprite = sprite.clone();
        activate_sprite(state, &sprite);
        return;
    }
    if has_wall_at(state, new_pos) {
        return;
    }

    state.set_pos(new_pos);
}

fn leave_room(state: &mut State, new_pos: bs::Position) -> bool {
    let room = &state.game.rooms[state.room];
    for exit in &room.exits {
        if exit.position != new_pos {
            continue;
        }
        let pos = exit.exit.position;
        let room_id = exit.exit.room_id.clone();
        if let Some(dialog_id) = &exit.dialogue_id {
            let dialog_id = dialog_id.clone();
            show_dialog(state, &dialog_id);
        }
        state.set_pos(pos);
        state.set_room(room_id);
        return true;
    }
    false
}

fn activate_sprite(state: &mut State, sprite: &bs::Sprite) {
    let dialog_id = match &sprite.dialogue_id {
        Some(id) => id,
        None => &sprite.id,
    };
    show_dialog(state, dialog_id)
}

fn show_dialog(state: &mut State, dialog_id: &str) {
    let Some(dialog) = state.game.dialogues.iter().find(|d| d.id == dialog_id) else {
        return;
    };
    if dialog.contents.trim().is_empty() {
        return;
    }
    let font = state.font.as_font();
    let char_width = font.char_width();
    let char_height = font.char_height();
    let lines = Dialog::new(
        &dialog.contents,
        &mut state.script_state,
        char_width,
        char_height,
    );
    state.dialog = lines;
}

fn get_avatar(state: &mut State) -> &mut bs::Sprite {
    for sprite in &mut state.game.sprites {
        if sprite.id == state.script_state.avatar {
            return sprite;
        }
    }
    panic!("avatar not found")
}

fn has_wall_at(state: &State, pos: bs::Position) -> bool {
    let Some(tile) = get_tile_at(state, pos) else {
        return false;
    };
    if tile.wall == Some(true) {
        return true;
    }
    let tile_id = tile.id.clone();
    let room = &state.game.rooms[state.room];
    if let Some(walls) = &room.walls {
        for wall in walls {
            if wall == &tile_id {
                return true;
            }
        }
    }
    false
}

fn get_sprite_at(state: &mut State, pos: bs::Position) -> Option<&bs::Sprite> {
    let room = &state.game.rooms[state.room];
    for sprite in &state.game.sprites {
        if sprite.id == "A" {
            continue;
        }
        let Some(sprite_room) = sprite.room_id.as_ref() else {
            continue;
        };
        if sprite_room != &room.id {
            continue;
        }
        if sprite.position != Some(pos) {
            continue;
        }
        return Some(sprite);
    }
    None
}

fn pop_item_at(state: &mut State, pos: bs::Position) -> Option<&bs::Item> {
    let idx = get_item_idx_at(state, pos)?;
    let room = &mut state.game.rooms[state.room];
    let item_ref = room.items.remove(idx);
    state.script_state.inventory.put(item_ref.id.clone());
    state.game.get_item(&item_ref.id)
}

fn get_item_idx_at(state: &mut State, pos: bs::Position) -> Option<usize> {
    let room = &state.game.rooms[state.room];
    for (i, item_ref) in room.items.iter().enumerate() {
        if item_ref.position == pos {
            return Some(i);
        }
    }
    None
}

fn get_tile_at(state: &State, pos: bs::Position) -> Option<&bs::Tile> {
    let room = &state.game.rooms[state.room];
    let idx = pos.y * TILES_X + pos.x;
    let tile_id = &room.tiles[usize::from(idx)];
    state.game.get_tile(tile_id)
}
