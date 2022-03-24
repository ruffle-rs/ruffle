use std::{borrow::Cow, ops::DerefMut};
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


const SWF : &'static [u8] = include_bytes!("z0r-de_6249.swf");
//const SWF : &'static [u8] = include_bytes!("z0r-de_420.swf");

async fn run(event_loop: EventLoop<()>, window: Window) {

    let mut time = Instant::now();
    let mut next_frame_time = Instant::now();

    let mut playerbox : Option<Arc<Mutex<Player>>> = None;


    log::info!("running eventloop");

    event_loop.run(move |event, _, control_flow| {
        //let _ = (&instance, &adapter, &shader, &pipeline_layout);

        //log::info!("RUFFLE EVENT: {:#?}", event);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                log::info!("resized");
                /*sc_desc.width = size.width;
                sc_desc.height = size.height;*/
                //swap_chain = Some(device.create_swap_chain(&surface.as_ref().unwrap(), &sc_desc));
            }
            Event::Resumed =>
            {
                log::info!("RUFFLE RESUMED");

                if playerbox.is_none() {

                    println!("resume");
                    let size = window.inner_size();

                    let renderer = Box::new(WgpuRenderBackend::for_window(
                        &window,
                        (1080, 1920),
                        wgpu::Backends::VULKAN,
                        wgpu::PowerPreference::HighPerformance,
                        None,
                    ).unwrap());


                    let start = std::time::Instant::now();
                    let log = Box::new(log_backend::NullLogBackend::new());
                    let audio = Box::new(CpalAudioBackend::new().unwrap());
                    let navigator = Box::new(NullNavigatorBackend::new());
                    let storage = Box::new(MemoryStorageBackend::default());
                    //let renderer = Box::new(NullRenderer::new());
                    let video = Box::new(SoftwareVideoBackend::new());
                    let ui = Box::new(NullUiBackend::new());



                    playerbox = Some(Player::new(renderer, audio, navigator, storage, video, log, ui).unwrap());

                    let mut player  = playerbox.as_ref().unwrap();

                    let mut player_lock = player.lock().unwrap();


                    let movie = SwfMovie::from_data(&SWF.to_vec(), None, None).unwrap();

                    player_lock.set_root_movie(Arc::new(movie));
                    player_lock.set_is_playing(true); // Desktop player will auto-play.

                    log::info!("MOVIE STARTED");


                }
                /*sc_desc.width = size.width;
                sc_desc.height = size.height;
                surface = Some(unsafe { instance.create_surface(&window) });*/
                //swap_chain = Some(device.create_swap_chain(&surface.as_ref().unwrap(), &sc_desc));
                //println!("surface: {:?}", surface);
            },
            Event::Suspended =>
            {
                println!("suspend");
                //swap_chain.take();
                //surface.take();
            },
            Event::MainEventsCleared => {


                let new_time = Instant::now();
                let dt = new_time.duration_since(time).as_micros();
                //log::info!("EVENTS CLEARED DT: {}", dt);
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
                    //window.request_redraw();

                }

            }

            // Render
            Event::RedrawRequested(_) => {
                log::info!("REDRAWING");
                // Don't render when minimized to avoid potential swap chain errors in `wgpu`.

                if playerbox.is_some() {
                    let mut player = playerbox.as_ref().unwrap();

                    let mut player_lock = player.lock().unwrap();
                    player_lock.render();
                    log::info!("RUFFLE RENDERED");
                }
                //
                log::info!("REDRAW DONE");
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
/*
const GET_DEVICES_OUTPUTS: jni::sys::jint = 2;

fn enumerate_audio_devices() -> Result<(), Box<dyn std::error::Error>> {


    log::info!("-- AUDIO QUERY --");


    // Create a VM for executing Java calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;

    // Query the global Audio Service
    let class_ctxt = env.find_class("android/content/Context")?;
    let audio_service = env.get_static_field(class_ctxt, "AUDIO_SERVICE", "Ljava/lang/String;")?;

    let audio_manager = env
        .call_method(
            ctx.context().cast(),
            "getSystemService",
            // JNI type signature needs to be derived from the Java API
            // (ArgTys)ResultTy
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[audio_service],
        )?
        .l()?;

    // Enumerate output devices
    let devices = env.call_method(
        audio_manager,
        "getDevices",
        "(I)[Landroid/media/AudioDeviceInfo;",
        &[GET_DEVICES_OUTPUTS.into()],
    )?;

    log::info!("-- Output Audio Devices --");

    let device_array = devices.l()?.into_inner();
    let len = env.get_array_length(device_array)?;
    for i in 0..len {
        let device = env.get_object_array_element(device_array, i)?;

        // Collect device information
        // See https://developer.android.com/reference/android/media/AudioDeviceInfo
        let product_name: String = {
            let name =
                env.call_method(device, "getProductName", "()Ljava/lang/CharSequence;", &[])?;
            let name = env.call_method(name.l()?, "toString", "()Ljava/lang/String;", &[])?;
            env.get_string(name.l()?.into())?.into()
        };
        let id = env.call_method(device, "getId", "()I", &[])?.i()?;
        let ty = env.call_method(device, "getType", "()I", &[])?.i()?;

        let sample_rates = {
            let sample_array = env
                .call_method(device, "getSampleRates", "()[I", &[])?
                .l()?
                .into_inner();
            let len = env.get_array_length(sample_array)?;

            let mut sample_rates = vec![0; len as usize];
            env.get_int_array_region(sample_array, 0, &mut sample_rates)?;
            sample_rates
        };

        log::info!("Device {}: Id {}, Type {}", product_name, id, ty);
        log::info!("sample rates: {:#?}", sample_rates);
    }

    Ok(())
}
*/
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on", logger(level = "info", tag = "ruffle")))]
fn main() {

    //enumerate_audio_devices().unwrap();

    log::info!("start");
    let event_loop = EventLoop::new();
    log::info!("got eventloop");
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.inner_size();
    log::info!("got window");

    pollster::block_on(run(event_loop, window));
}
