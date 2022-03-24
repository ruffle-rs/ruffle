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


//const SWF : &'static [u8] = include_bytes!("z0r-de_6249.swf");
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
                        wgpu::Backends::all(),
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


fn get_swf_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {


    log::info!("-- GETTING SWF CONTENTS --");


    // Create a VM for executing Java calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;



    let intent = env.call_method(ctx.context() as jni::sys::jobject, "getIntent", "()Landroid/content/Intent;", &[])?;



    log::info!("GOT INTENT");


    let extras = env.call_method(intent.l()?, "getExtras", "()Landroid/os/Bundle;", &[])?;


    log::info!("GOT EXTRAS");

    let uri = env.call_method(extras.l()?, "get", "(Ljava/lang/String;)Ljava/lang/Object;", &[jni::objects::JValue::Object(env.new_string("SWF_URI")?.into())])?;

    log::info!("GOT URI");

    let resolver = env.call_method(ctx.context() as jni::sys::jobject, "getContentResolver", "()Landroid/content/ContentResolver;", &[])?;

    log::info!("GOT RESOLVER");

    let stream = env.call_method(resolver.l()?, "openInputStream", "(Landroid/net/Uri;)Ljava/io/InputStream;", &[jni::objects::JValue::Object(uri.l()?)])?;

    log::info!("GOT STREAM");

    let available = env.call_method(stream.l()?, "available", "()I", &[])?;

    log::info!("GOT AVAILABLE: {:}", available.i()?);

    let bytes = env.new_byte_array(available.i()?)?;

    log::info!("MADE ARRAY");

    let num_bytes_read = env.call_method(stream.l()?, "read", "([B)I", &[jni::objects::JValue::Object(jni::objects::JObject::from(bytes))])?;

    log::info!("READ BYTES: {:}", num_bytes_read.i()?);



    let elements = env.get_byte_array_elements(bytes as jbyteArray, ReleaseMode::NoCopyBack)?;

    log::info!("GOT ELEMENTS");

    unsafe {
        Ok(Vec::from_raw_parts(elements.as_ptr() as *mut u8, elements.size()? as usize, elements.size()? as usize))
    }



/*****************************************
    let intent = env.call_method(ctx.context() as jni::sys::jobject, "getIntent", "()Landroid/content/Intent;", &[])?;



    log::info!("GOT INTENT");


    let extras = env.call_method(intent.l()?, "getExtras", "()Landroid/os/Bundle;", &[])?;


    log::info!("GOT EXTRAS");

    let bytearray = env.call_method(extras.l()?, "getByteArray", "(Ljava/lang/String;)[B", &[jni::objects::JValue::Object(env.new_string("SWF_BYTES")?.into())])?;

    log::info!("GOT FIELD");

    let elements = env.get_byte_array_elements(bytearray.l()?.into_inner() as jbyteArray, ReleaseMode::NoCopyBack)?;

    log::info!("GOT ELEMENTS");

    unsafe {
        Ok(Vec::from_raw_parts(elements.as_ptr() as *mut u8, elements.size()? as usize, elements.size()? as usize))
    }

********************************/






/*************************


    let class_intent = env.find_class("android/content/Intent")?;
    let ACTION_GET_CONTENT = env.get_static_field(class_intent, "ACTION_GET_CONTENT", "Ljava/lang/String;")?;
    let intent = env.new_object(class_intent, "(Ljava/lang/String;)V", &[ACTION_GET_CONTENT])?;


    log::info!("INTENT CREATED");
    env.call_method(intent, "setType", "(Ljava/lang/String;)Landroid/content/Intent;", &[jni::objects::JValue::Object(env.new_string("application/x-shockwave-flash")?.into())])?;
    log::info!("TYPE SET");


    //intent.addCategory(Intent.CATEGORY_OPENABLE);

    let activity_class = env.call_method(ctx.context() as jni::sys::jobject, "getClass", "()Ljava/lang/Class;", &[])?.l()?;

    let activity_class_name = env.call_method(activity_class, "getName", "()Ljava/lang/String;", &[])?.l()?;

    log::info!("activity class name: {:#?}", activity_class_name);
    let str = env.get_string_utf_chars(activity_class_name.into())?;
    let mut str2 = str;
    unsafe {
    while *str2 != 0 {
        str2 = str2.add(1);
    }
    let sl = std::slice::from_raw_parts(str, str2.offset_from(str) as usize);
    let s = String::from_utf8(sl.to_vec()).unwrap();
    log::info!("REAL CLASS NAME: {:#?}", s);
    }
    env.release_string_utf_chars(activity_class_name.into(), str);


    // here's hoping our Context is an Activity
    env.call_method(ctx.context() as jni::sys::jobject, "startActivityForResult", "(Landroid/content/Intent;I)V", &[jni::objects::JValue::Object(intent), jni::objects::JValue::Int(0)])?;


*************************/


/*

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
    */
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
