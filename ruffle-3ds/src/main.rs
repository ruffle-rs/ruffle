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

static SWF_BYTES: &[u8] = include_bytes!("../swf/sample.swf");

fn create_player(gfx: Gfx) -> Arc<Mutex<Player>> {
    let movie = SwfMovie::from_data(SWF_BYTES, "file:///".to_string(), None).unwrap();
    println!("got movie");
    let renderer = render::Citro3DRenderBackend::new(gfx).expect("man i really fucked up early");

    PlayerBuilder::new()
        .with_movie(movie)
        .with_renderer(renderer)
        .with_viewport_dimensions(400, 240, 1.0)
        .build()
}

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
        if hid.keys_down().contains(KeyPad::A) {
            let time = SystemTime::now();
            let dt = time
                .duration_since(prev_time)
                .expect("failed to read time")
                .as_millis();
            prev_time = time;

            player_lock.tick(1000.0 / 60.0 /*dt as f64*/);
            player_lock.render();
        }

        let renderer: &mut render::Citro3DRenderBackend =
            player_lock.renderer_mut().downcast_mut().unwrap();
        renderer.gfx_mut().wait_for_vblank();
    }
}

// //! This example demonstrates the most basic usage of `citro3d`: rendering a simple
// //! RGB triangle (sometimes called a "Hello triangle") to the 3DS screen.
//
// #![feature(allocator_api)]
//
// use citro3d::macros::include_shader;
// use citro3d::math::{AspectRatio, ClipPlanes, Matrix4, Projection, StereoDisplacement};
// use citro3d::render::{ClearFlags, Frame, ScreenTarget, Target};
// use citro3d::texenv;
// use citro3d::{attrib, buffer, shader};
// use ctru::prelude::*;
// use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen3D};
//
// #[repr(C)]
// #[derive(Copy, Clone)]
// struct Vec2 {
//     x: f32,
//     y: f32,
//     z: f32,
// }
//
// impl Vec2 {
//     const fn new(x: f32, y: f32, z: f32) -> Self {
//         Self { x, y, z }
//     }
// }
//
// #[repr(C)]
// #[derive(Copy, Clone)]
// struct Vertex {
//     pos: Vec2,
//     color: Vec2,
// }
//
// static VERTICES: &[Vertex] = &[
//     Vertex {
//         pos: Vec2::new(0.0, 0.5, -3.0),
//         color: Vec2::new(1.0, 0.0, 0.0),
//     },
//     Vertex {
//         pos: Vec2::new(-0.5, -0.5, -3.0),
//         color: Vec2::new(0.0, 1.0, 0.0),
//     },
//     Vertex {
//         pos: Vec2::new(0.5, -0.5, -3.0),
//         color: Vec2::new(0.0, 0.0, 1.0),
//     },
// ];
//
// static SHADER_BYTES: &[u8] = include_shader!("../shaders/color.v.pica");
// const CLEAR_COLOR: u32 = 0x68_B0_D8_FF;
//
// fn main() {
//     let mut soc = Soc::new().expect("failed to get SOC");
//     drop(soc.redirect_to_3dslink(true, true));
//
//     let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
//     let mut hid = Hid::new().expect("Couldn't obtain HID controller");
//     let apt = Apt::new().expect("Couldn't obtain APT controller");
//
//     let mut instance = citro3d::Instance::new().expect("failed to initialize Citro3D");
//
//     let top_screen = TopScreen3D::from(&gfx.top_screen);
//
//     let (mut top_left, mut top_right) = top_screen.split_mut();
//
//     let RawFrameBuffer { width, height, .. } = top_left.raw_framebuffer();
//     let mut top_left_target = instance
//         .render_target(width, height, top_left, None)
//         .expect("failed to create render target");
//
//     let RawFrameBuffer { width, height, .. } = top_right.raw_framebuffer();
//     let mut top_right_target = instance
//         .render_target(width, height, top_right, None)
//         .expect("failed to create render target");
//
//     let mut bottom_screen = gfx.bottom_screen.borrow_mut();
//     let RawFrameBuffer { width, height, .. } = bottom_screen.raw_framebuffer();
//
//     let mut bottom_target = instance
//         .render_target(width, height, bottom_screen, None)
//         .expect("failed to create bottom screen render target");
//
//     let shader = shader::Library::from_bytes(SHADER_BYTES).unwrap();
//     let vertex_shader = shader.get(0).unwrap();
//
//     let program = shader::Program::new(vertex_shader).unwrap();
//     let projection_uniform_idx = program.get_uniform("projection").unwrap();
//
//     let vbo_data = buffer::Buffer::new(VERTICES);
//
//     let mut buf_info = buffer::Info::new();
//     let attr_info = prepare_vbos(&mut buf_info, vbo_data);
//
//     let stage0 = texenv::TexEnv::new()
//         .src(texenv::Mode::BOTH, texenv::Source::PrimaryColor, None, None)
//         .func(texenv::Mode::BOTH, texenv::CombineFunc::Replace);
//
//     while apt.main_loop() {
//         hid.scan_input();
//
//         if hid.keys_down().contains(KeyPad::START) {
//             break;
//         }
//
//         instance.render_frame_with(|mut frame| {
//             // Sadly closures can't have lifetime specifiers,
//             // so we wrap `render_to` in this function to force the borrow checker rules.
//             fn cast_lifetime_to_closure<'frame, T>(x: T) -> T
//             where
//                 T: Fn(&mut Frame<'frame>, &'frame mut ScreenTarget<'_>, &Matrix4),
//             {
//                 x
//             }
//
//             let render_to = cast_lifetime_to_closure(|frame, target, projection| {
//                 target.clear(ClearFlags::ALL, CLEAR_COLOR, 0);
//
//                 frame
//                     .select_render_target(target)
//                     .expect("failed to set render target");
//                 frame.bind_vertex_uniform(projection_uniform_idx, projection);
//
//                 frame.set_texenvs(&[stage0]);
//
//                 frame.set_attr_info(&attr_info);
//
//                 frame
//                     .draw_arrays(buffer::Primitive::Triangles, &buf_info, None)
//                     .unwrap();
//             });
//
//             // We bind the vertex shader.
//             frame.bind_program(&program);
//
//             // Configure the first fragment shading substage to just pass through the vertex color
//             // See https://www.opengl.org/sdk/docs/man2/xhtml/glTexEnv.xml for more insight
//
//             let Projections {
//                 left_eye,
//                 right_eye,
//                 center,
//             } = calculate_projections();
//
//             render_to(&mut frame, &mut top_left_target, &left_eye);
//             render_to(&mut frame, &mut top_right_target, &right_eye);
//             render_to(&mut frame, &mut bottom_target, &center);
//
//             frame
//         });
//     }
// }
//
// fn prepare_vbos(buf_info: &mut buffer::Info, vbo_data: buffer::Buffer) -> attrib::Info {
//     // Configure attributes for use with the vertex shader
//     let mut attr_info = attrib::Info::new();
//
//     attr_info
//         .add_loader(attrib::Register::V0, attrib::Format::Float, 3)
//         .unwrap();
//
//     attr_info
//         .add_loader(attrib::Register::V1, attrib::Format::Float, 3)
//         .unwrap();
//
//     buf_info.add(vbo_data, attr_info.permutation()).unwrap();
//
//     attr_info
// }
//
// struct Projections {
//     left_eye: Matrix4,
//     right_eye: Matrix4,
//     center: Matrix4,
// }
//
// fn calculate_projections() -> Projections {
//     // TODO: it would be cool to allow playing around with these parameters on
//     // the fly with D-pad, etc.
//     let slider_val = ctru::os::current_3d_slider_state();
//     let interocular_distance = slider_val / 2.0;
//
//     let vertical_fov = 40.0_f32.to_radians();
//     let screen_depth = 2.0;
//
//     let clip_planes = ClipPlanes {
//         near: 0.01,
//         far: 100.0,
//     };
//
//     let (left, right) = StereoDisplacement::new(interocular_distance, screen_depth);
//
//     let (left_eye, right_eye) =
//         Projection::perspective(vertical_fov, AspectRatio::TopScreen, clip_planes)
//             .stereo_matrices(left, right);
//
//     let center =
//         Projection::perspective(vertical_fov, AspectRatio::BottomScreen, clip_planes).into();
//
//     Projections {
//         left_eye,
//         right_eye,
//         center,
//     }
// }
