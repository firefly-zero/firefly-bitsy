use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use bitsy_reparser as bs;
use firefly_rust as ff;

const TILES_X: u8 = 16;
const TILES_Y: u8 = 16;
const OFFSET_X: i32 = (ff::WIDTH - 8 * 16) / 2;
const OFFSET_Y: i32 = 0;

pub fn render_room(state: &mut State) {
    if state.script_state.end && state.dialog.n_pages() == 0 {
        draw_end(state);
        return;
    }
    if !state.script_state.end {
        clear_room(state);
        set_palette(state);
        draw_tiles(state);
        draw_items(state);
        draw_sprites(state);
        draw_avatar(state);
    }
    draw_dialog(state);
}

fn draw_end(state: &State) {
    ff::clear_screen(ff::Color::Black);
    let font = state.font.as_font();
    let x = (ff::WIDTH - i32::from(font.char_width()) * 7) / 2;
    let y = (ff::HEIGHT + i32::from(font.char_height())) / 2;
    let point = ff::Point::new(x, y);
    ff::draw_text("THE END", &font, point, ff::Color::Purple);
}

fn set_palette(state: &State) {
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
    for (i, tile) in &state.tiles {
        // if let Some(color) = tile.colour_id {
        //     let color = Color::try_from(color as usize).unwrap();
        // }
        let frame = pick_frame(&tile.animation_frames, state.frame);
        let primary = match tile.colour_id {
            Some(c) => c as u8,
            None => 1,
        };
        let image = parse_image(frame, primary);
        let image = unsafe { ff::Image::from_bytes(&image) };
        let x = i % TILES_X;
        let y = i / TILES_Y;
        let point = tile_point(x, y);
        ff::draw_image(&image, point);
    }
}

fn clear_room(state: &State) {
    let color = ff::Color::Black;
    if state.dialog.n_pages() == 0 {
        ff::clear_screen(color);
    }
    let point = ff::Point::new(OFFSET_X, OFFSET_Y);
    let size = ff::Size::new(TILES_X * 8, TILES_Y * 8);
    ff::draw_rect(point, size, ff::Style::solid(color));
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
        let primary = match item.colour_id {
            Some(c) => c as u8,
            None => 2,
        };
        let image = parse_image(frame, primary);
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
        if sprite.id == state.script_state.avatar {
            draw_sprite(sprite, state.frame);
            return;
        }
    }
}

fn draw_sprite(sprite: &bs::Sprite, frame: u8) {
    let frame = pick_frame(&sprite.animation_frames, frame);
    let Some(pos) = &sprite.position else {
        return;
    };
    let primary = match sprite.colour_id {
        Some(c) => c as u8,
        None => 2,
    };
    let image = parse_image(frame, primary);
    let image = unsafe { ff::Image::from_bytes(&image) };
    let point = tile_point(pos.x, pos.y);
    ff::draw_image(&image, point);
}

fn draw_dialog(state: &mut State) {
    const MARGIN_X: i32 = 2;

    let center = state.dialog.center;
    let Some(page) = state.dialog.current_page() else {
        return;
    };
    // Slow down word rendering.
    if !page.fast && !state.frame.is_multiple_of(3) {
        return;
    }

    let y = if center { 128 / 2 } else { OFFSET_Y + 128 };
    let point = ff::Point::new(0, y);
    if !page.started {
        page.started = true;
        let color = ff::Color::Black;
        if center {
            ff::clear_screen(color);
        } else {
            let size = ff::Size::new(ff::WIDTH, ff::HEIGHT - point.y);
            let style = ff::Style::solid(color);
            ff::draw_rect(point, size, style);
        }
    }

    let font = state.font.as_font();
    let point = ff::Point::new(point.x + MARGIN_X, point.y + 10);
    for word in &mut page.words {
        use bitsy_script::Word::*;
        match &word.word {
            Text(text, _) => {
                if word.rendered {
                    continue;
                }
                word.rendered = true;
                let word_point = point + word.point;
                ff::draw_text(text, &font, word_point, ff::Color::Purple);
                if !page.fast {
                    return;
                }
            }
            Sprite(_) => {}
            Tile(_) => {}
            Item(_) => {}
            LineBreak => {}
            PageBreak => {}
        };
    }
    if state.dialog.n_pages() > 1 {
        draw_dialog_arrow()
    }
}

fn draw_dialog_arrow() {
    ff::draw_triangle(
        ff::Point::new(229, 153),
        ff::Point::new(229 + 8, 153),
        ff::Point::new(229 + 4, 153 + 4),
        ff::Style::solid(ff::Color::LightGray),
    );
}

fn parse_image(image: &bs::Image, primary: u8) -> Vec<u8> {
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
    for i in 0..image.pixels.len() / 2 {
        let p1 = image.pixels[i * 2] * primary;
        let p2 = image.pixels[i * 2 + 1] * primary;
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
