use std::{borrow::Cow, ops::DerefMut};
use jni::sys::jbyteArray;
use jni::objects::ReleaseMode;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

mod audio;

use audio::CpalAudioBackend;

use ruffle_core::{
    backend::{
        log as log_backend,
        navigator::NullNavigatorBackend,
        storage::MemoryStorageBackend,
        ui::NullUiBackend,
        render::NullRenderer,
        video::SoftwareVideoBackend,
    },
    Player,
    tag_utils::SwfMovie,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::time::Instant;

async fn run(event_loop: EventLoop<()>, window: Window) {

    let mut time = Instant::now();
    let mut next_frame_time = Instant::now();

    let mut playerbox : Option<Arc<Mutex<Player>>> = None;

    log::info!("running eventloop");

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                log::info!("resized");
            }
            Event::Resumed =>
            {
                println!("resume");
                log::info!("RUFFLE RESUMED");

                if playerbox.is_none() {

                    let size = window.inner_size();

                    let renderer = Box::new(WgpuRenderBackend::for_window(
                        &window,
                        (1080, 1920),
                        wgpu::Backends::all(),
                        wgpu::PowerPreference::HighPerformance,
                        None,
                    ).unwrap());

                    let start = std::time::Instant::now();
                    let log = Box::new(log_backend::NullLogBackend::new());
                    let audio = Box::new(CpalAudioBackend::new().unwrap());
                    let navigator = Box::new(NullNavigatorBackend::new());
                    let storage = Box::new(MemoryStorageBackend::default());
                    let video = Box::new(SoftwareVideoBackend::new());
                    let ui = Box::new(NullUiBackend::new());

                    playerbox = Some(Player::new(renderer, audio, navigator, storage, video, log, ui).unwrap());

                    let mut player  = playerbox.as_ref().unwrap();
                    let mut player_lock = player.lock().unwrap();

                    match get_swf_bytes() {
                        Ok(bytes) => {
                            let movie = SwfMovie::from_data(&bytes, None, None).unwrap();

                            player_lock.set_root_movie(Arc::new(movie));
                            player_lock.set_is_playing(true); // Desktop player will auto-play.

                            log::info!("MOVIE STARTED");

                        },
                        Err(e) => {
                            log::error!("{}", e);
                        }

                    }
                }
            },
            Event::Suspended =>
            {
                println!("suspend");
            },
            Event::MainEventsCleared => {
                let new_time = Instant::now();
                let dt = new_time.duration_since(time).as_micros();

                if dt > 0 {
                    time = new_time;
                    if playerbox.is_some() {
                        let mut player = playerbox.as_ref().unwrap();

                        let mut player_lock = player.lock().unwrap();
                        player_lock.tick(dt as f64 / 1000.0);
                        //log::info!("RUFFLE TICKED");
                        next_frame_time = new_time + player_lock.time_til_next_frame();

                        if player_lock.needs_render() {
                            window.request_redraw();
                            log::info!("REQUESTED REDRAW");
                        }
                    }
                }

            }

            // Render
            Event::RedrawRequested(_) => {
                log::info!("REDRAWING");
                // TODO: Don't render when minimized to avoid potential swap chain errors in `wgpu`.
                // TODO: also disable when suspended!

                if playerbox.is_some() {
                    let mut player = playerbox.as_ref().unwrap();

                    let mut player_lock = player.lock().unwrap();
                    player_lock.render();
                    log::info!("RUFFLE RENDERED");
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            _ => {}
        }


        // After polling events, sleep the event loop until the next event or the next frame.
        if *control_flow != ControlFlow::Exit {
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
        }

    });
}


fn get_swf_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a VM for executing Java calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;

    // The following is basically equivalent to this Java code:
    /*
    Intent intent = getIntent();
    Bundle extras = intent.getExtras();
    Uri uri = extras.get("SWF_URI");

    ContentResolver resolver = getContentResolver();
    InputStream inputStream = resolver.openInputStream(uri);

    int available = inputStream.available();
    byte[] bytes = new byte[available];
    // assuming the whole contents will be available at once
    int _num_bytes_read = inputStream.read(bytes);
    */

    let intent = env.call_method(ctx.context() as jni::sys::jobject, "getIntent", "()Landroid/content/Intent;", &[])?;
    let extras = env.call_method(intent.l()?, "getExtras", "()Landroid/os/Bundle;", &[])?;
    let uri = env.call_method(extras.l()?, "get", "(Ljava/lang/String;)Ljava/lang/Object;", &[jni::objects::JValue::Object(env.new_string("SWF_URI")?.into())])?;

    let resolver = env.call_method(ctx.context() as jni::sys::jobject, "getContentResolver", "()Landroid/content/ContentResolver;", &[])?;
    let stream = env.call_method(resolver.l()?, "openInputStream", "(Landroid/net/Uri;)Ljava/io/InputStream;", &[jni::objects::JValue::Object(uri.l()?)])?;

    let available = env.call_method(stream.l()?, "available", "()I", &[])?;
    let bytes = env.new_byte_array(available.i()?)?;
    let _num_bytes_read = env.call_method(stream.l()?, "read", "([B)I", &[jni::objects::JValue::Object(jni::objects::JObject::from(bytes))])?;

    // And finally getting the bytes into a Vec
    let elements = env.get_byte_array_elements(bytes as jbyteArray, ReleaseMode::NoCopyBack)?;
    unsafe {
        Ok(Vec::from_raw_parts(elements.as_ptr() as *mut u8, elements.size()? as usize, elements.size()? as usize))
    }
}

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on", logger(level = "info", tag = "ruffle")))]
fn main() {

    log::info!("start");
    let event_loop = EventLoop::new();
    log::info!("got eventloop");
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.inner_size();
    log::info!("got window");

    pollster::block_on(run(event_loop, window));
}
