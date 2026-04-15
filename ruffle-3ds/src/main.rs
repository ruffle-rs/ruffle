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
    }
}

// // This example demonstrates the most basic usage of `citro3d`: rendering a simple
// // RGB triangle (sometimes called a "Hello triangle") to the 3DS screen.
//
// #![feature(allocator_api)]
//
// use citro3d::macros::include_shader;
// use citro3d::math::{AspectRatio, ClipPlanes, FVec4, Matrix4, Projection, StereoDisplacement};
// use citro3d::render::{ClearFlags, Frame, ScreenTarget, Target};
// use citro3d::texenv;
// use citro3d::{attrib, buffer, shader};
// use ctru::linear::LinearAllocator;
// use ctru::prelude::*;
// use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen3D};
// use ruffle_render::backend::null::NullBitmapSource;
// use ruffle_render::backend::RenderBackend;
// use ruffle_render::commands::{CommandHandler, CommandList};
// use ruffle_render::matrix::Matrix;
// use ruffle_render::shape_utils::{DistilledShape, DrawCommand, DrawPath, FillRule};
// use ruffle_render::transform::Transform;
// use swf::{Color, ColorTransform, FillStyle, Point, Rectangle, Twips};
//
// mod render;
//
// #[repr(C)]
// #[derive(Copy, Clone)]
// struct Vec2 {
//     x: f32,
//     y: f32,
// }
//
// impl Vec2 {
//     const fn new(x: f32, y: f32) -> Self {
//         Self { x, y }
//     }
// }
//
// #[repr(C)]
// #[derive(Copy, Clone)]
// struct Vertex {
//     pos: Vec2,
//     color: [f32; 4],
// }
//
// static VERTICES: &[Vertex] = &[
//     Vertex {
//         pos: Vec2::new(0.0, 0.5),
//         color: [1.0, 0.0, 0.0, 1.0],
//     },
//     Vertex {
//         pos: Vec2::new(-0.5, -0.5),
//         color: [0.0, 1.0, 0.0, 1.0],
//     },
//     Vertex {
//         pos: Vec2::new(0.5, -0.5),
//         color: [0.0, 0.0, 1.0, 1.0],
//     },
// ];
//
// // static SHADER_BYTES: &[u8] = include_shader!("../shaders/color.v.pica");
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
//     ctru::set_panic_hook(true);
//
//     // let gfx_copy = std::mem::ManuallyDrop::new(unsafe { std::ptr::read(&gfx) });
//     // let _console = Console::new(gfx_copy.bottom_screen.borrow_mut());
//     // basic init done
//
//     println!("starting");
//     let mut renderer = render::Citro3DRenderBackend::new(gfx).unwrap();
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
//     let mut world_matrix = Matrix4::identity();
//     let mut multclr = FVec4::new(1.0, 0.0, 0.0, 1.0);
//     let mut addclr = FVec4::new(0.0, 0.0, 0.0, 0.0);
//
//     let indices = [0u16, 1, 2].to_vec_in(LinearAllocator);
//
//     while apt.main_loop() {
//         hid.scan_input();
//
//         if hid.keys_down().contains(KeyPad::START) {
//             break;
//         }
//
//         if hid.keys_down().contains(KeyPad::DPAD_UP) {
//             world_matrix.translate(0.0, 0.0, 5.0);
//             println!("{:?}", world_matrix);
//         }
//         if hid.keys_down().contains(KeyPad::DPAD_DOWN) {
//             world_matrix.translate(0.0, 0.0, -5.0);
//             println!("{:?}", world_matrix);
//         }
//         if hid.keys_down().contains(KeyPad::DPAD_LEFT) {
//             multclr = multclr + FVec4::new(0.0, 0.1, 0.1, 0.0);
//         }
//         if hid.keys_down().contains(KeyPad::DPAD_RIGHT) {
//             multclr = multclr + FVec4::new(0.0, -0.1, -0.1, 0.0);
//         }
//
//         if !hid.keys_down().contains(KeyPad::A) {
//             continue;
//         }
//
//         let projections = calculate_projections();
//         renderer.submit_vbo(
//             Color::from_rgba(CLEAR_COLOR),
//             &buf_info,
//             &attr_info,
//             &indices,
//             world_matrix,
//             projections.center,
//             multclr,
//             addclr,
//         );
//
//         // let mut commands = CommandList::new();
//         // commands.render_shape(
//         //     shape.clone(),
//         //     Transform {
//         //         matrix: Matrix::translate(Twips::from_pixels(50.0), Twips::from_pixels(50.0)),
//         //         color_transform: ColorTransform::default(),
//         //     },
//         // );
//         // renderer.submit_frame(Color::CYAN, commands, vec![]);
//     }
// }
//
// fn triangle_distilled_shape() -> DistilledShape<'static> {
//     // Define vertices (in twips; 20 twips = 1 pixel)
//     let top = Point::new(Twips::from_pixels(100.0), Twips::from_pixels(0.0));
//     let left = Point::new(Twips::from_pixels(0.0), Twips::from_pixels(200.0));
//     let right = Point::new(Twips::from_pixels(200.0), Twips::from_pixels(200.0));
//
//     static FILL: FillStyle = FillStyle::Color(Color {
//         r: 255,
//         g: 0,
//         b: 0,
//         a: 255,
//     });
//
//     let commands = vec![
//         DrawCommand::MoveTo(top),
//         DrawCommand::LineTo(right),
//         DrawCommand::LineTo(left),
//         DrawCommand::LineTo(top), // close the path
//     ];
//
//     let bounds = Rectangle {
//         x_min: Twips::from_pixels(0.0),
//         x_max: Twips::from_pixels(200.0),
//         y_min: Twips::from_pixels(0.0),
//         y_max: Twips::from_pixels(200.0),
//     };
//
//     DistilledShape {
//         paths: vec![DrawPath::Fill {
//             style: &FILL,
//             commands,
//             winding_rule: FillRule::EvenOdd,
//         }],
//         shape_bounds: bounds.clone(),
//         edge_bounds: bounds,
//         id: 1,
//     }
// }
//
// fn prepare_vbos(buf_info: &mut buffer::Info, vbo_data: buffer::Buffer) -> attrib::Info {
//     // Configure attributes for use with the vertex shader
//     let mut attr_info = attrib::Info::new();
//
//     attr_info
//         .add_loader(attrib::Register::V0, attrib::Format::Float, 2)
//         .unwrap();
//
//     attr_info
//         .add_loader(attrib::Register::V1, attrib::Format::Float, 4)
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
//
// // #![feature(allocator_api)]
// //
// // use citro3d::macros::include_shader;
// // use citro3d::math::{AspectRatio, ClipPlanes, FVec4, Matrix4, Projection, StereoDisplacement};
// // use citro3d::render::{ClearFlags, Frame, ScreenTarget, Target};
// // use citro3d::shader::Program;
// // use citro3d::{attrib, buffer, shader};
// // use citro3d::{color, texenv};
// // use ctru::linear::LinearAllocator;
// // use ctru::prelude::*;
// // use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen3D};
// // use ruffle_render::backend::null::NullBitmapSource;
// // use ruffle_render::backend::RenderBackend;
// // use ruffle_render::commands::{CommandHandler, CommandList};
// // use ruffle_render::matrix::Matrix;
// // use ruffle_render::shape_utils::{DistilledShape, DrawCommand, DrawPath, FillRule};
// // use ruffle_render::transform::Transform;
// // use swf::{Color, ColorTransform, FillStyle, Point, Rectangle, Twips};
// //
// // use crate::render::ShaderProgram;
// //
// // mod render;
// //
// // #[repr(C)]
// // #[derive(Copy, Clone)]
// // struct Vec2 {
// //     x: f32,
// //     y: f32,
// // }
// //
// // impl Vec2 {
// //     const fn new(x: f32, y: f32) -> Self {
// //         Self { x, y }
// //     }
// // }
// //
// // #[repr(C)]
// // #[derive(Copy, Clone)]
// // struct Vertex {
// //     pos: Vec2,
// //     color: [f32; 4],
// // }
// //
// // static VERTICES: &[Vertex] = &[
// //     Vertex {
// //         pos: Vec2::new(0.0, 0.5),
// //         color: [1.0, 0.0, 0.0, 1.0],
// //     },
// //     Vertex {
// //         pos: Vec2::new(-0.5, -0.5),
// //         color: [0.0, 1.0, 0.0, 1.0],
// //     },
// //     Vertex {
// //         pos: Vec2::new(0.5, -0.5),
// //         color: [0.0, 0.0, 1.0, 1.0],
// //     },
// // ];
// //
// // // static SHADER_BYTES: &[u8] = include_shader!("../shaders/color.v.pica");
// // const CLEAR_COLOR: u32 = 0x68_B0_D8_FF;
// //
// // fn main() {
// //     // let mut soc = Soc::new().expect("failed to get SOC");
// //     // drop(soc.redirect_to_3dslink(true, true));
// //
// //     let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
// //     let mut hid = Hid::new().expect("Couldn't obtain HID controller");
// //     let apt = Apt::new().expect("Couldn't obtain APT controller");
// //
// //     let gfx_copy = std::mem::ManuallyDrop::new(unsafe { std::ptr::read(&gfx) });
// //     let _console = Console::new(gfx_copy.bottom_screen.borrow_mut());
// //     // basic init done
// //
// //     let mut bottom_screen = gfx.bottom_screen.borrow_mut();
// //     let RawFrameBuffer { width, height, .. } = bottom_screen.raw_framebuffer();
// //
// //     // let mut instance = citro3d::Instance::new().expect("failed to initialize Citro3D");
// //     //
// //     // let mut bottom_target = instance
// //     //     .render_target(width, height, bottom_screen, None)
// //     //     .expect("failed to create bottom screen render target");
// //     //
// //     // let shader = shader::Library::from_bytes(include_shader!("../shaders/color.v.pica")).unwrap();
// //     // let vertex_shader = shader.get(0).unwrap();
// //     //
// //     // let program = shader::Program::new(vertex_shader).unwrap();
// //     // let world_u = program.get_uniform("world").unwrap();
// //     // let view_u = program.get_uniform("view").unwrap();
// //     // let mult_u = program.get_uniform("multclr").unwrap();
// //     // let add_u = program.get_uniform("addclr").unwrap();
// //     //
// //     // println!("got unifs");
// //
// //     // =======================================================
// //     let mut instance = citro3d::Instance::new().unwrap();
// //
// //     let color_shader = shader::Library::from_bytes(include_shader!("../shaders/color.v.pica"))
// //         .expect("failed to load color shader");
// //
// //     let color_prog = shader::Program::new(color_shader.get(0).unwrap())
// //         .expect("failed to create color shader program");
// //
// //     let color_stage0 = texenv::TexEnv::new()
// //         .src(texenv::Mode::BOTH, texenv::Source::PrimaryColor, None, None)
// //         .func(texenv::Mode::BOTH, texenv::CombineFunc::Replace);
// //     let color_program = render::ShaderProgram::new(color_prog, color_stage0);
// //
// //     // let texture_shader =
// //     // shader::Library::from_bytes(include_shader!("../shaders/texture.v.pica"))
// //     //     .expect("failed to load texture shader");
// //     // let texture_prog = Program::new(texture_shader.get(0).unwrap())
// //     //     .expect("failed to create texture shader program")
// //     //     .into();
// //     // let texture_stage0 = TexEnv::new().src(texenv::Mode::RGB, texenv::Source::Texture0, None, None);
// //     // let texture_program = ShaderProgram::new(texture_prog, texture_stage0);
// //
// //     // attributes for vertex buffer is always the same for both shaders, so we store this as a field
// //     let mut attr_info = attrib::Info::new();
// //
// //     // v0 (position) = Float Vec2
// //     attr_info
// //         .add_loader(attrib::Register::V0, attrib::Format::Float, 2)
// //         .unwrap();
// //     // v1 (color) = Float Vec4
// //     attr_info
// //         .add_loader(attrib::Register::V1, attrib::Format::Float, 4)
// //         .unwrap();
// //     // =======================================================
// //
// //     let vbo_data = buffer::Buffer::new(VERTICES);
// //
// //     let mut buf_info = buffer::Info::new();
// //     let attr_info = prepare_vbos(&mut buf_info, vbo_data);
// //
// //     let stage0 = texenv::TexEnv::new()
// //         .src(texenv::Mode::BOTH, texenv::Source::PrimaryColor, None, None)
// //         .func(texenv::Mode::BOTH, texenv::CombineFunc::Replace);
// //
// //     let mut world_matrix = Matrix4::identity();
// //     let mut multclr = FVec4::new(1.0, 0.0, 0.0, 1.0);
// //     let mut addclr = FVec4::new(0.0, 0.0, 0.0, 0.0);
// //
// //     let indices = [0u16, 1, 2].to_vec_in(LinearAllocator);
// //
// //     while apt.main_loop() {
// //         hid.scan_input();
// //
// //         if hid.keys_down().contains(KeyPad::START) {
// //             break;
// //         }
// //
// //         if hid.keys_down().contains(KeyPad::DPAD_UP) {
// //             world_matrix.translate(0.0, 0.0, 5.0);
// //             println!("{:?}", world_matrix);
// //         }
// //         if hid.keys_down().contains(KeyPad::DPAD_DOWN) {
// //             world_matrix.translate(0.0, 0.0, -5.0);
// //             println!("{:?}", world_matrix);
// //         }
// //         if hid.keys_down().contains(KeyPad::DPAD_LEFT) {
// //             multclr = multclr + FVec4::new(0.0, 0.1, 0.1, 0.0);
// //         }
// //         if hid.keys_down().contains(KeyPad::DPAD_RIGHT) {
// //             multclr = multclr + FVec4::new(0.0, -0.1, -0.1, 0.0);
// //         }
// //
// //         if !hid.keys_down().contains(KeyPad::A) {
// //             continue;
// //         }
// //
// //         let mut top_screen = gfx.top_screen.borrow_mut();
// //         let RawFrameBuffer { width, height, .. } = top_screen.raw_framebuffer();
// //
// //         let mut top_target = instance
// //             .render_target(width, height, top_screen, None)
// //             .expect("failed to create target");
// //
// //         let projection = calculate_projections().center;
// //         instance.render_frame_with(|mut frame| {
// //             top_target.clear(ClearFlags::ALL, CLEAR_COLOR, 0);
// //
// //             frame.select_render_target(&top_target);
// //
// //             // println!("rendering shape");
// //             //
// //             // // ==================
// //             // let program = &color_program.program;
// //             // // ==================
// //             //
// //             // // bind shader program
// //             // frame.bind_program(&program);
// //             //
// //             // println!("world: {:?}", world_matrix);
// //             // println!("worldView: {:?}", projection * world_matrix);
// //             //
// //             // // // bind uniforms
// //             // // frame.bind_vertex_uniform(view_u, projection);
// //             // // frame.bind_vertex_uniform(world_u, world_matrix);
// //             // // frame.bind_vertex_uniform(mult_u, multclr);
// //             // // frame.bind_vertex_uniform(add_u, addclr);
// //             // frame.bind_vertex_uniform(color_program.view_idx, projection);
// //             // frame.bind_vertex_uniform(color_program.world_idx, world_matrix);
// //             // frame.bind_vertex_uniform(color_program.multclr_idx, multclr);
// //             // frame.bind_vertex_uniform(color_program.addclr_idx, addclr);
// //             //
// //             // frame.set_texenvs(&[stage0]);
// //             // frame.set_attr_info(&attr_info);
// //             //
// //             // frame.draw_elements(buffer::Primitive::Triangles, &buf_info, &indices);
// //
// //             render_to(
// //                 &color_program,
// //                 &attr_info,
// //                 &mut frame,
// //                 &buf_info,
// //                 &indices,
// //                 world_matrix,
// //                 projection,
// //                 multclr,
// //                 addclr,
// //             );
// //
// //             println!("drew elements");
// //
// //             frame
// //         });
// //     }
// // }
// //
// // fn render_to<'a>(
// //     color_program: &'a ShaderProgram,
// //     attr: &attrib::Info,
// //     frame: &mut Frame<'a>,
// //     info: &'a buffer::Info,
// //     idxs: &'a Vec<u16, LinearAllocator>,
// //     world: Matrix4,
// //     proj: Matrix4,
// //     m: FVec4,
// //     a: FVec4,
// // ) {
// //     // ==================
// //     let program = &color_program.program;
// //     // ==================
// //
// //     // bind shader program
// //     frame.bind_program(&program);
// //
// //     println!("world: {:?}", world);
// //     println!("worldView: {:?}", proj * world);
// //
// //     // // bind uniforms
// //     // frame.bind_vertex_uniform(view_u, projection);
// //     // frame.bind_vertex_uniform(world_u, world_matrix);
// //     // frame.bind_vertex_uniform(mult_u, multclr);
// //     // frame.bind_vertex_uniform(add_u, addclr);
// //     frame.bind_vertex_uniform(color_program.view_idx, proj);
// //     frame.bind_vertex_uniform(color_program.world_idx, world);
// //     frame.bind_vertex_uniform(color_program.multclr_idx, m);
// //     frame.bind_vertex_uniform(color_program.addclr_idx, a);
// //
// //     frame.set_texenvs(&[color_program.texenv]);
// //     frame.set_attr_info(&attr);
// //
// //     frame.draw_elements(buffer::Primitive::Triangles, &info, &idxs);
// // }
// //
// // fn prepare_vbos(buf_info: &mut buffer::Info, vbo_data: buffer::Buffer) -> attrib::Info {
// //     // Configure attributes for use with the vertex shader
// //     let mut attr_info = attrib::Info::new();
// //
// //     attr_info
// //         .add_loader(attrib::Register::V0, attrib::Format::Float, 2)
// //         .unwrap();
// //
// //     attr_info
// //         .add_loader(attrib::Register::V1, attrib::Format::Float, 4)
// //         .unwrap();
// //
// //     buf_info.add(vbo_data, attr_info.permutation()).unwrap();
// //
// //     attr_info
// // }
// //
// // struct Projections {
// //     left_eye: Matrix4,
// //     right_eye: Matrix4,
// //     center: Matrix4,
// // }
// //
// // fn calculate_projections() -> Projections {
// //     // TODO: it would be cool to allow playing around with these parameters on
// //     // the fly with D-pad, etc.
// //     let slider_val = ctru::os::current_3d_slider_state();
// //     let interocular_distance = slider_val / 2.0;
// //
// //     let vertical_fov = 40.0_f32.to_radians();
// //     let screen_depth = 2.0;
// //
// //     let clip_planes = ClipPlanes {
// //         near: 0.01,
// //         far: 100.0,
// //     };
// //
// //     let (left, right) = StereoDisplacement::new(interocular_distance, screen_depth);
// //
// //     let (left_eye, right_eye) =
// //         Projection::perspective(vertical_fov, AspectRatio::TopScreen, clip_planes)
// //             .stereo_matrices(left, right);
// //
// //     let center =
// //         Projection::perspective(vertical_fov, AspectRatio::BottomScreen, clip_planes).into();
// //
// //     Projections {
// //         left_eye,
// //         right_eye,
// //         center,
// //     }
// // }
