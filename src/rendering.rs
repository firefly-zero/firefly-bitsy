use crate::*;
use alloc::vec;
use alloc::vec::Vec;
use firefly_rust::{self as ff, RGB};

const TILES_X: u8 = 16;
const TILES_Y: u8 = 16;
const OFFSET_X: i32 = (ff::WIDTH - 8 * 16) / 2;
const OFFSET_Y: i32 = 0;
const ANIMATION_DELAY: u16 = 25;

const COLOR_BG: ff::Color = ff::Color::new(1);
const COLOR_RAINBOW: ff::Color = ff::Color::LightGreen;
const COLOR_DIALOG_BOX: ff::Color = ff::Color::Gray;
const COLOR_DIALOG_TEXT: ff::Color = ff::Color::DarkGray;

const RAINBOW_COLORS: &[ff::RGB] = &[
    ff::RGB::new(255, 0, 0),   // red
    ff::RGB::new(255, 217, 0), // yellow
    ff::RGB::new(78, 255, 0),  // green
    ff::RGB::new(0, 255, 125), // also green
    ff::RGB::new(0, 192, 255), // sky blue
    ff::RGB::new(0, 18, 255),  // blue
    ff::RGB::new(136, 0, 255), // purple
    ff::RGB::new(255, 0, 242), // pink
    ff::RGB::new(255, 0, 138), // also pink
    ff::RGB::new(255, 0, 61),  // also red
];

pub fn render_room(state: &mut State) {
    let render_frame = state.update_frame / ANIMATION_DELAY;
    let new_frame = state.render_frame != render_frame;
    state.render_frame = render_frame;

    if !state.segments.is_empty() {
        draw_progress_bar(state);
        return;
    }

    if state.script_state.end && state.dialog.n_pages() == 0 {
        draw_end(state);
        return;
    }
    let render_room = !state.script_state.end && (new_frame | state.room_dirty);
    if render_room {
        state.room_dirty = false;
        clear_room(state);
        set_palette(state);
        draw_tiles(state);
        draw_items(state);
        draw_sprites(state);
        draw_avatar(state);
    }
    draw_dialog(state);
}

fn draw_progress_bar(state: &State) {
    const TEXT: &str = "LOADING SCRIPT...";
    ff::clear_screen(ff::Color::Black);
    let font = state.font.as_font();
    let x = (ff::WIDTH - i32::from(font.char_width()) * TEXT.len() as i32) / 2;
    let y = (ff::HEIGHT + i32::from(font.char_height())) / 2;
    let point = ff::Point::new(x, y);
    ff::draw_text(TEXT, &font, point, ff::Color::Gray);

    if state.n_segments != 0 {
        let segments_left = state.n_segments - state.segments.len();
        let progress = TEXT.len() * segments_left / state.n_segments;
        ff::draw_text(&TEXT[..progress], &font, point, ff::Color::White);
    }
}

/// Render "THE END" screen.
fn draw_end(state: &State) {
    ff::clear_screen(COLOR_DIALOG_BOX);
    let font = state.font.as_font();
    let x = (ff::WIDTH - i32::from(font.char_width()) * 7) / 2;
    let y = (ff::HEIGHT + i32::from(font.char_height())) / 2;
    let point = ff::Point::new(x, y);
    ff::draw_text("THE END", &font, point, COLOR_DIALOG_TEXT);
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
        let rgb = convert_color(color);
        ff::set_color(idx, rgb);
    }

    // If the base palette colors are contrast enough,
    // use them for the dialog box as well.
    // It's usually true but some games can play around with palette.
    // For example, to have "hidden" tiles in a room.
    if palette.colours.len() >= 2 {
        let bg = convert_color(&palette.colours[0]);
        let fg = convert_color(&palette.colours[1]);
        if is_contrast(bg, fg) {
            ff::set_color(COLOR_DIALOG_BOX, bg);
            ff::set_color(COLOR_DIALOG_TEXT, fg);
            return;
        }
    };

    ff::set_color(COLOR_DIALOG_BOX, RGB::new(0x21, 0x1e, 0x20));
    ff::set_color(COLOR_DIALOG_TEXT, RGB::new(0xe9, 0xef, 0xec));
}

fn convert_color(c: &bitsy_file::Colour) -> ff::RGB {
    ff::RGB {
        r: c.red,
        g: c.green,
        b: c.blue,
    }
}

fn draw_tiles(state: &State) {
    for (i, images) in &state.tiles {
        let image = pick_raw_frame(images, state.render_frame);
        let image = unsafe { ff::Image::from_bytes(image) };
        let x = i % TILES_X;
        let y = i / TILES_Y;
        let point = tile_point(x, y);
        ff::draw_image(&image, point);
    }
}

fn clear_room(state: &State) {
    if state.dialog.n_pages() == 0 {
        ff::clear_screen(COLOR_BG);
    }
    let point = ff::Point::new(OFFSET_X, OFFSET_Y);
    let size = ff::Size::new(TILES_X * 8, TILES_Y * 8);
    ff::draw_rect(point, size, ff::Style::solid(COLOR_BG));
}

fn draw_items(state: &State) {
    let room = &state.game.rooms[state.room];
    for item in &room.items {
        let pos = &item.position;
        let id = &item.id;
        let Some(item) = state.game.items.iter().find(|item| &item.id == id) else {
            continue;
        };
        let frame = pick_frame(&item.animation_frames, state.render_frame);
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
            draw_sprite(sprite, state.render_frame);
        }
    }
}

fn draw_avatar(state: &State) {
    for sprite in &state.game.sprites {
        if sprite.id == state.script_state.avatar {
            draw_sprite(sprite, state.render_frame);
            return;
        }
    }
}

fn draw_sprite(sprite: &bitsy_file::Sprite, frame: u16) {
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
    if !page.fast && !state.update_frame.is_multiple_of(3) {
        return;
    }

    let y = if center { 128 / 2 } else { OFFSET_Y + 128 };
    let point = ff::Point::new(0, y);
    if !page.started {
        page.started = true;
        if center {
            ff::clear_screen(COLOR_BG);
        }
        let size = ff::Size::new(ff::WIDTH, 32);
        let style = ff::Style::solid(COLOR_DIALOG_BOX);
        ff::draw_rect(point, size, style);
    }

    // Cycle the RGB representation of the color representing the rainbow text.
    let idx = usize::from(state.update_frame / 6) % RAINBOW_COLORS.len();
    let rainbow_color = RAINBOW_COLORS[idx];
    ff::set_color(COLOR_RAINBOW, rainbow_color);

    let font = state.font.as_font();
    let point = ff::Point::new(point.x + MARGIN_X, point.y + 10);
    for word in &mut page.words {
        use bitsy_script::Word::*;
        match &word.word {
            Text(text, effect) => {
                use bitsy_script::TextEffect::*;
                let apply_effect = state.update_frame.is_multiple_of(3);
                let stable = matches!(effect, None | Color(_));
                if word.rendered && (!apply_effect || stable) {
                    continue;
                }
                let mut word_point = point + word.point;
                let mut color = COLOR_DIALOG_TEXT;
                let mut wave = false;

                // If the text effect moves the word around,
                // hide the old word first.
                let moving = matches!(effect, Wavy | Shaky);
                if moving {
                    let width = font.line_width(text) as i32;
                    let height = i32::from(font.char_height());
                    ff::draw_rect(
                        ff::Point::new(word_point.x, word_point.y - 6),
                        ff::Size::new(width, height + 1),
                        ff::Style::solid(COLOR_DIALOG_BOX),
                    );
                }

                // Change text parameters to apply the text effect.
                {
                    match effect {
                        None => {}
                        Wavy => wave = true,
                        Shaky => {
                            let rand = ff::get_random();
                            let shift_x = rand % 2 - 1;
                            let shift_y = (rand >> 8) % 2 - 1;
                            word_point.x += shift_x as i32;
                            word_point.y += shift_y as i32;
                        }
                        Rainbow => color = COLOR_RAINBOW,
                        Color(c) => color = ff::Color::new(*c),
                    }
                }

                if wave {
                    // Draw the wavy word letter-by-letter.
                    for i in 0..text.len() {
                        let sub = &text[i..=i];
                        let shift_x = (i * usize::from(font.char_width())) as i32;
                        let shift_y = ((state.render_frame + i as u16) % 2) as i32;
                        let point = word_point + ff::Point::new(shift_x, shift_y);
                        ff::draw_text(sub, &font, point, color);
                    }
                } else {
                    ff::draw_text(text, &font, word_point, color);
                }

                let was_rendered = word.rendered;
                word.rendered = true;
                if !was_rendered && !page.fast {
                    return;
                }
            }
            Sprite(_) => {}
            Tile(_) => {}
            Item(_) => {}
            LineBreak | PageBreak => {}
        };
    }
    if state.dialog.n_pages() > 1 {
        draw_dialog_arrow(state)
    }
}

fn draw_dialog_arrow(state: &State) {
    let y = if state.dialog.center { 89 } else { 153 };
    ff::draw_triangle(
        ff::Point::new(229, y),
        ff::Point::new(229 + 8, y),
        ff::Point::new(229 + 4, y + 4),
        ff::Style::solid(COLOR_DIALOG_TEXT),
    );
}

pub fn parse_image(image: &bitsy_file::Image, primary: u8) -> Vec<u8> {
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

fn pick_frame(frames: &[bitsy_file::Image], frame: u16) -> &bitsy_file::Image {
    let frame = usize::from(frame);
    &frames[frame % frames.len()]
}

fn pick_raw_frame(frames: &[Image], frame: u16) -> &Image {
    let frame = usize::from(frame);
    &frames[frame % frames.len()]
}

/// Check if the given colors have a high contrast ratio.
fn is_contrast(c1: ff::RGB, c2: ff::RGB) -> bool {
    let l1 = luminance(c1);
    let l2 = luminance(c2);
    // https://www.accessibility-developer-guide.com/knowledge/colours-and-contrast/how-to-calculate/
    let mut contrast = (l1 + 0.05) / (l2 + 0.05);
    if contrast < 1.0 {
        contrast = 1.0 / contrast;
    }
    // The contrast values lie on the range from 1 to 21
    // where 1 is the same color and 21 is #FFF and #000.
    //
    // I've picked 10 as the threshold by playing with online contrast calculators
    // and picking the minimum that still looks good. But 7 would also be ok.
    contrast >= 10.0
}

fn luminance(c: ff::RGB) -> f32 {
    // https://www.w3.org/TR/WCAG20/#relativeluminancedef
    let r = srgb_linear(c.r);
    let g = srgb_linear(c.g);
    let b = srgb_linear(c.b);
    (r * 0.2126) + (g * 0.7152) + (b * 0.0722)
}

fn srgb_linear(v: u8) -> f32 {
    // https://www.w3.org/TR/WCAG20/#relativeluminancedef
    let v: f32 = f32::from(v) / 255.;
    if v <= 0.04045 {
        v / 12.92
    } else {
        let x = (v + 0.055) / 1.055;
        // The original formula uses x^2.4 (powf) but we don't have `powf`.
        // on no-std environment. So here's a close approximation.
        // I've made a plot for the range and it good enough.
        1.28 * x * x - 0.28 * x
    }
}
