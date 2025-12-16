use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use bitsy_nostd_parser as bs;
use firefly_rust as ff;

const TILES_X: i32 = 16;
const TILES_Y: i32 = 16;
const OFFSET_X: i32 = (ff::WIDTH - 8 * 16) / 2;
const OFFSET_Y: i32 = 0;

pub fn render_room(state: &State) {
    set_palette(state);
    draw_tiles(state);
    draw_items(state);
    draw_sprites(state);
    draw_avatar(state);
    draw_dialog(state);
}

fn set_palette(state: &State) {
    ff::clear_screen(ff::Color::Black);
    let room = &state.game.rooms[state.room];
    let palette = match &room.palette_id {
        Some(id) => id.as_str(),
        None => "0",
    };
    let palette = state.game.get_palette(palette).unwrap();
    for (color, idx) in palette.colours.iter().zip(1_usize..) {
        let idx = ff::Color::from(idx as u8);
        let rgb = ff::RGB {
            r: color.red,
            g: color.green,
            b: color.blue,
        };
        ff::set_color(idx, rgb);
    }
}

fn draw_tiles(state: &State) {
    let room = &state.game.rooms[state.room];
    for (tile_id, i) in room.tiles.iter().zip(0..) {
        let Some(tile) = &state.game.get_tile(tile_id) else {
            continue;
        };
        // if let Some(color) = tile.colour_id {
        //     let color = Color::try_from(color as usize).unwrap();
        // }
        let frame = pick_frame(&tile.animation_frames, state.frame);
        let image = parse_image(frame, false);
        let image = unsafe { ff::Image::from_bytes(&image) };
        let x = (i % TILES_X) as u8;
        let y = (i / TILES_Y) as u8;
        let point = tile_point(x, y);
        ff::draw_image(&image, point);
    }
}

fn draw_items(state: &State) {
    let room = &state.game.rooms[state.room];
    for item in &room.items {
        let pos = &item.position;
        let id = &item.id;
        let Some(item) = state.game.items.iter().find(|item| &item.id == id) else {
            continue;
        };
        let frame = pick_frame(&item.animation_frames, state.frame);
        let image = parse_image(frame, true);
        let image = unsafe { ff::Image::from_bytes(&image) };
        let point = tile_point(pos.x, pos.y);
        ff::draw_image(&image, point);
    }
}

fn draw_sprites(state: &State) {
    let room = &state.game.rooms[state.room];
    for sprite in &state.game.sprites {
        if sprite.id == "A" {
            continue;
        }
        let Some(room_id) = sprite.room_id.as_ref() else {
            continue;
        };
        if room_id == &room.id {
            draw_sprite(sprite, state.frame);
        }
    }
}

fn draw_avatar(state: &State) {
    for sprite in &state.game.sprites {
        if sprite.id == "A" {
            draw_sprite(sprite, state.frame);
        }
    }
}

fn draw_sprite(sprite: &bs::Sprite, frame: u8) {
    let frame = pick_frame(&sprite.animation_frames, frame);
    let Some(pos) = &sprite.position else {
        return;
    };
    let image = parse_image(frame, true);
    let image = unsafe { ff::Image::from_bytes(&image) };
    let point = tile_point(pos.x, pos.y);
    ff::draw_image(&image, point);
}

fn draw_dialog(state: &State) {
    let Some(dialog) = &state.dialog else {
        return;
    };

    let point = ff::Point::new(0, OFFSET_Y + 128);
    let size = ff::Size::new(ff::WIDTH, ff::HEIGHT - point.y);
    let style = ff::Style::solid(ff::Color::White);
    ff::draw_rect(point, size, style);

    let font = state.font.as_font();
    let point = ff::Point::new(point.x + 4, point.y + 10);
    let text = &dialog;
    ff::draw_text(text, &font, point, ff::Color::DarkGray);
}

fn parse_image(image: &bs::Image, sprite: bool) -> Vec<u8> {
    let pixels = &image.pixels;
    let is_hd = pixels.len() == 256;
    let width = if is_hd { 16 } else { 8 };
    let height = width;

    const HEADER_SIZE: usize = 5 + 8;
    let body_size = width * height / 2;
    let mut raw = vec![0; HEADER_SIZE + body_size as usize];

    // Header.
    raw[0] = 0x21; // magic number
    raw[1] = 4; // BPP
    raw[2] = width as u8; // width
    raw[3] = (width >> 8) as u8; // width
    raw[4] = 255; // transparency
    // color swaps
    for i in 0u8..8u8 {
        raw[5 + i as usize] = ((i * 2) << 4) | (i * 2 + 1);
    }

    let mult = if sprite { 2 } else { 1 };
    for i in 0..image.pixels.len() / 2 {
        let p1 = image.pixels[i * 2] * mult;
        let p2 = image.pixels[i * 2 + 1] * mult;
        raw[HEADER_SIZE + i] = p1 << 4 | p2;
    }

    raw
}

fn tile_point(x: u8, y: u8) -> ff::Point {
    let x = OFFSET_X + i32::from(x) * 8;
    let y = OFFSET_Y + i32::from(y) * 8;
    ff::Point::new(x, y)
}

fn pick_frame(frames: &[bs::Image], frame: u8) -> &bs::Image {
    let frame = usize::from(frame / 12);
    &frames[frame % frames.len()]
}
