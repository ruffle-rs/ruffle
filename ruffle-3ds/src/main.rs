#![feature(allocator_api)]

use ctru::prelude::*;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, ViewportDimensions};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

mod log;
mod render;

const DIMENSIONS: ViewportDimensions = ViewportDimensions {
    width: 400,
    height: 240,
    scale_factor: 1.0,
};

static SWF_BYTES: &[u8] = include_bytes!("../swf/logo-anim.swf");

fn create_player(gfx: Gfx) -> Arc<Mutex<Player>> {
    let movie = SwfMovie::from_data(SWF_BYTES, "file:///".to_string(), None).unwrap();
    println!("got movie");

    let renderer = render::Citro3DRenderBackend::new(gfx).expect("man i really fucked up early");
    println!("made renderer");

    PlayerBuilder::new()
        .with_movie(movie)
        .with_renderer(renderer)
        .with_viewport_dimensions(400, 240, 1.0)
        .build()
}

// fn create_player() -> Arc<Mutex<Player>> {
//     let movie = SwfMovie::from_data(SWF_BYTES, "file:///".to_string(), None).unwrap();
//     println!("got movie");
//
//     let renderer = log::LogRenderer::new(DIMENSIONS);
//     println!("made renderer");
//
//     PlayerBuilder::new()
//         .with_movie(movie)
//         .with_renderer(renderer)
//         .with_viewport_dimensions(400, 240, 1.0)
//         .build()
// }

fn main() {
    let gfx = Gfx::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let apt = Apt::new().unwrap();

    let gfx_copy = std::mem::ManuallyDrop::new(unsafe { std::ptr::read(&gfx) });
    let _console = Console::new(gfx_copy.bottom_screen.borrow_mut());

    // let mut soc = Soc::new().unwrap();
    // soc.redirect_to_3dslink(true, true);

    ctru::set_panic_hook(true);

    println!("creating player");

    let player = create_player(gfx);
    player.lock().unwrap().set_is_playing(true);

    println!("player started");
    println!("press A to advance frame");

    let mut prev_time = SystemTime::now();

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        let mut player_lock = player.lock().unwrap();
        if true {
            //hid.keys_down().contains(KeyPad::A) {
            let time = SystemTime::now();
            let dt = time
                .duration_since(prev_time)
                .expect("failed to read time")
                .as_millis();
            prev_time = time;

            player_lock.tick(1000.0 / 60.0); //dt as f64);
            player_lock.render();
        }
    }
}
