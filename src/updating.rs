use crate::*;
use firefly_rust as ff;

const TILES_X: u8 = 16;
const TILES_Y: u8 = 16;

pub fn update_state(state: &mut State) {
    state.frame = (state.frame + 1) % 60;
    handle_pad(state);
}

fn handle_pad(state: &mut State) {
    let dpad = match ff::read_pad(ff::Peer::COMBINED) {
        Some(pad) => pad.as_dpad(),
        None => ff::DPad::default(),
    };
    let pressed = dpad.just_pressed(&state.dpad);
    state.dpad = dpad;
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

fn move_avatar_to(state: &mut State, dx: i8, dy: i8) {
    let avatar = get_avatar(state);
    let Some(old_pos) = &avatar.position else {
        return;
    };
    let x = old_pos.x.saturating_add_signed(dx).min(TILES_X - 1);
    let y = old_pos.y.saturating_add_signed(dy).min(TILES_Y - 1);
    let new_pos = bs::Position { x, y };
    let Some(room_id) = &avatar.room_id else {
        return;
    };
    let room_id = room_id.clone();

    if let Some(sprite) = get_sprite_at(state, &room_id, new_pos) {
        return;
    }

    let avatar = get_avatar(state);
    avatar.position = Some(new_pos);
}

fn get_avatar(state: &mut State) -> &mut bs::Sprite {
    for sprite in &mut state.game.sprites {
        if &sprite.id == "A" {
            return sprite;
        }
    }
    panic!("avatar not found")
}

fn get_sprite_at<'a>(
    state: &'a mut State,
    room: &str,
    pos: bs::Position,
) -> Option<&'a bs::Sprite> {
    for sprite in &state.game.sprites {
        let Some(sprite_room) = sprite.room_id.as_ref() else {
            continue;
        };
        if sprite_room != room {
            continue;
        }
        if sprite.position != Some(pos) {
            continue;
        }
        return Some(sprite);
    }
    None
}
