#![no_std]
#![no_main]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use bitsy_nostd_parser as bs;
use core::cell::OnceCell;
use firefly_rust as ff;

static mut STATE: OnceCell<State> = OnceCell::new();
const OFFSET_X: i32 = (ff::WIDTH - 8 * 16) / 2;
const OFFSET_Y: i32 = (ff::HEIGHT - 8 * 16) / 2;

struct State {
    game: bs::Game,
    room: usize,
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
    let state = State { game, room: 0 };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    // ...
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    set_palette(state);
    draw_tiles(state);
    draw_avatar(state);
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
        let Ok(tile) = &state.game.get_tile_by_id(tile_id) else {
            continue;
        };
        // if let Some(color) = tile.colour_id {
        //     let color = Color::try_from(color as usize).unwrap();
        // }
        let frame = &tile.animation_frames[0];
        let image = parse_image(frame);
        let image = unsafe { ff::Image::from_bytes(&image) };
        let x = OFFSET_X + (i % 16) * 8;
        let y = OFFSET_Y + (i / 16) * 8;
        let point = ff::Point::new(x, y);
        ff::draw_image(&image, point);
    }
}

fn draw_avatar(state: &State) {
    let sprite = state.game.get_avatar().unwrap();
    let frame = &sprite.animation_frames[0];
    let Some(pos) = &sprite.position else {
        return;
    };
    let image = parse_image(frame);
    let image = unsafe { ff::Image::from_bytes(&image) };
    let x = OFFSET_X + i32::from(pos.x) * 8;
    let y = OFFSET_Y + i32::from(pos.y) * 8;
    let point = ff::Point::new(x, y);
    ff::draw_image(&image, point);
}

fn parse_image(image: &bs::Image) -> Vec<u8> {
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
        let p1 = image.pixels[i * 2];
        let p2 = image.pixels[i * 2 + 1];
        raw[HEADER_SIZE + i] = p1 << 4 | p2;
    }

    raw
}
