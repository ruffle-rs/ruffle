use crate::util::fs_commands::{FsCommand, TestFsCommandProvider};
use crate::util::image_trigger::ImageTrigger;
use crate::util::navigator::TestNavigatorBackend;
use crate::util::options::ImageComparison;
use crate::util::test::Test;
use anyhow::{anyhow, Result};
use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, DecodeError, RegisterError, SoundHandle, SoundInstanceHandle,
    SoundTransform,
};
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::NullExecutor;
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::events::{KeyCode, TextControlCode as RuffleTextControlCode};
use ruffle_core::impl_audio_mixer_backend;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{
    AutomatedEvent, InputInjector, MouseButton as InputMouseButton,
    TextControlCode as InputTextControlCode,
};
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_socket_format::SocketEvent;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct TestAudioBackend {
    mixer: AudioMixer,
    buffer: Vec<f32>,
}

impl TestAudioBackend {
    const NUM_CHANNELS: u8 = 2;
    const SAMPLE_RATE: u32 = 44100;

    pub fn new() -> Self {
        Self {
            mixer: AudioMixer::new(Self::NUM_CHANNELS, Self::SAMPLE_RATE),
            buffer: vec![],
        }
    }
}

impl AudioBackend for TestAudioBackend {
    impl_audio_mixer_backend!(mixer);
    fn play(&mut self) {}
    fn pause(&mut self) {}

    fn set_frame_rate(&mut self, frame_rate: f64) {
        let new_buffer_size =
            ((Self::NUM_CHANNELS as u32 * Self::SAMPLE_RATE) as f64 / frame_rate).round() as usize;
        self.buffer.resize(new_buffer_size, 0.0);
    }
    fn tick(&mut self) {
        debug_assert!(!self.buffer.is_empty());
        self.mixer.mix::<f32>(self.buffer.as_mut());
    }
}

#[derive(Clone)]
pub struct TestLogBackend {
    trace_output: Rc<RefCell<String>>,
}

impl TestLogBackend {
    pub fn new() -> Self {
        Self {
            trace_output: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn trace_output(self) -> String {
        self.trace_output.take()
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push_str(message);
        self.trace_output.borrow_mut().push('\n');
    }
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
pub fn run_swf(
    test: &Test,
    mut injector: InputInjector,
    socket_events: Option<Vec<SocketEvent>>,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
) -> Result<String> {
    let base_path = Path::new(&test.output_path).parent().unwrap();
    let mut executor = NullExecutor::new();
    let movie = SwfMovie::from_path(&test.swf_path, None).map_err(|e| anyhow!(e.to_string()))?;
    let mut frame_time = 1000.0 / movie.frame_rate().to_f64();
    if let Some(tr) = test.options.tick_rate {
        frame_time = tr;
    }

    let frame_time_duration = Duration::from_millis(frame_time as u64);

    let log = TestLogBackend::new();
    let (fs_command_provider, fs_commands) = TestFsCommandProvider::new();
    let navigator = TestNavigatorBackend::new(
        base_path,
        &executor,
        socket_events,
        test.options.log_fetch.then(|| log.clone()),
    )?;

    let builder = PlayerBuilder::new()
        .with_log(log.clone())
        .with_navigator(navigator)
        .with_max_execution_duration(Duration::from_secs(300))
        .with_fs_commands(Box::new(fs_command_provider))
        .with_viewport_dimensions(
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
            1.0,
        );

    // Test player options may override anything set above
    let player = test
        .options
        .player_options
        .setup(builder, &movie)?
        .with_movie(movie)
        .with_autoplay(true) //.tick() requires playback
        .build();

    let mut images = test.options.image_comparisons.clone();

    let wgpu_descriptors = if cfg!(feature = "imgtests") && !images.is_empty() {
        crate::util::environment::wgpu_descriptors()
    } else {
        None
    };

    before_start(player.clone())?;

    if test.options.num_frames.is_none() && test.options.num_ticks.is_none() {
        return Err(anyhow!(
            "Test {} must specify at least one of num_frames or num_ticks",
            test.swf_path.to_string_lossy()
        ));
    }

    let mut remaining_iterations = test
        .options
        .num_frames
        .or(test.options.num_ticks)
        .expect("valid iteration count");
    let mut current_iteration = 0;

    while remaining_iterations > 0 {
        // If requested, ensure that the 'expected' amount of
        // time actually elapses between frames. This is useful for
        // tests that call 'flash.utils.getTimer()' and use
        // 'setInterval'/'flash.utils.Timer'
        //
        // Note that when Ruffle actually runs frames, we can
        // execute frames faster than this in order to 'catch up'
        // if we've fallen behind. However, in order to make regression
        // tests deterministic, we always call 'update_timers' with
        // an elapsed time of 'frame_time'. By sleeping for 'frame_time_duration',
        // we ensure that the result of 'flash.utils.getTimer()' is consistent
        // with timer execution (timers will see an elapsed time of *at least*
        // the requested timer interval).
        if test.options.sleep_to_meet_frame_rate {
            std::thread::sleep(frame_time_duration);
        }

        while !player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {}

        if test.options.num_ticks.is_some() {
            player.lock().unwrap().tick(frame_time);
        } else {
            player.lock().unwrap().run_frame();
            player.lock().unwrap().update_timers(frame_time);
            player.lock().unwrap().audio_mut().tick();
        }
        remaining_iterations -= 1;
        current_iteration += 1;
        executor.run();

        for command in fs_commands.try_iter() {
            match command {
                FsCommand::Quit => {
                    remaining_iterations = 0;
                }
                FsCommand::CaptureImage(name) => {
                    if let Some(image_comparison) = images.remove(&name) {
                        if image_comparison.trigger != ImageTrigger::FsCommand {
                            return Err(anyhow!("Encountered fscommand to capture and compare image '{name}', but the trigger was expected to be {:?}", image_comparison.trigger));
                        }
                        capture_and_compare_image(
                            base_path,
                            &player,
                            wgpu_descriptors,
                            &name,
                            image_comparison,
                            test.options.known_failure,
                        )?;
                    } else {
                        return Err(anyhow!("Encountered fscommand to capture and compare image '{name}', but no [image_comparison] was set up for this."));
                    }
                }
            }
        }

        injector.next(|evt, _btns_down| {
            player.lock().unwrap().handle_event(match evt {
                AutomatedEvent::MouseDown { pos, btn } => PlayerEvent::MouseDown {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::MouseMove { pos } => PlayerEvent::MouseMove { x: pos.0, y: pos.1 },
                AutomatedEvent::MouseUp { pos, btn } => PlayerEvent::MouseUp {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::KeyDown { key_code } => PlayerEvent::KeyDown {
                    key_code: KeyCode::from_u8(*key_code).expect("Invalid keycode in test"),
                    key_char: None,
                },
                AutomatedEvent::TextInput { codepoint } => PlayerEvent::TextInput {
                    codepoint: *codepoint,
                },
                AutomatedEvent::TextControl { code } => PlayerEvent::TextControl {
                    code: match code {
                        InputTextControlCode::MoveLeft => RuffleTextControlCode::Backspace,
                        InputTextControlCode::MoveRight => RuffleTextControlCode::Delete,
                        InputTextControlCode::SelectLeft => RuffleTextControlCode::SelectLeft,
                        InputTextControlCode::SelectRight => RuffleTextControlCode::SelectRight,
                        InputTextControlCode::SelectAll => RuffleTextControlCode::SelectAll,
                        InputTextControlCode::Copy => RuffleTextControlCode::Copy,
                        InputTextControlCode::Paste => RuffleTextControlCode::Paste,
                        InputTextControlCode::Cut => RuffleTextControlCode::Cut,
                        InputTextControlCode::Backspace => RuffleTextControlCode::Backspace,
                        InputTextControlCode::Enter => RuffleTextControlCode::Enter,
                        InputTextControlCode::Delete => RuffleTextControlCode::Delete,
                    },
                },
                AutomatedEvent::Wait => unreachable!(),
            });
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        player.lock().unwrap().render();

        if let Some(name) = images
            .iter()
            .find(|(_k, v)| v.trigger == ImageTrigger::SpecificIteration(current_iteration))
            .map(|(k, _v)| k.to_owned())
        {
            let image_comparison = images
                .remove(&name)
                .expect("Name was just retrieved from map, should not be missing!");
            capture_and_compare_image(
                base_path,
                &player,
                wgpu_descriptors,
                &name,
                image_comparison,
                test.options.known_failure,
            )?;
        }
    }

    if let Some(name) = images
        .iter()
        .find(|(_k, v)| v.trigger == ImageTrigger::LastFrame)
        .map(|(k, _v)| k.to_owned())
    {
        let image_comparison = images
            .remove(&name)
            .expect("Name was just retrieved from map, should not be missing!");

        capture_and_compare_image(
            base_path,
            &player,
            wgpu_descriptors,
            &name,
            image_comparison,
            test.options.known_failure,
        )?;
    }

    if !images.is_empty() {
        return Err(anyhow!(
            "Image comparisons didn't trigger: {:?}",
            images.keys()
        ));
    }

    before_end(player)?;

    executor.run();

    let trace = log.trace_output();
    // Null bytes are invisible, and interfere with constructing
    // the expected output.txt file. Any tests dealing with null
    // bytes should explicitly test for them in ActionScript.
    let normalized_trace = trace.replace('\0', "");
    Ok(normalized_trace)
}

#[cfg(not(feature = "imgtests"))]
fn capture_and_compare_image(
    _base_path: &Path,
    _player: &Arc<Mutex<Player>>,
    _wgpu_descriptors: Option<&Arc<Descriptors>>,
    _name: &String,
    _image_comparison: ImageComparison,
    known_failure: bool,
) -> Result<()> {
    if known_failure {
        // It's possible that the trace output matched but the image might not.
        // If we aren't checking the image, pretend the match failed (which makes it actually pass, since it's expecting failure).
        return Err(anyhow!(
            "Not checking images, pretending this failed since we don't know if it worked."
        ));
    } else {
        Ok(())
    }
}

#[cfg(feature = "imgtests")]
fn capture_and_compare_image(
    base_path: &Path,
    player: &Arc<Mutex<Player>>,
    wgpu_descriptors: Option<&Arc<Descriptors>>,
    name: &String,
    image_comparison: ImageComparison,
    known_failure: bool,
) -> Result<()> {
    use anyhow::Context;
    use ruffle_render_wgpu::backend::WgpuRenderBackend;
    use ruffle_render_wgpu::target::TextureTarget;

    if let Some(wgpu_descriptors) = wgpu_descriptors {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();
        let renderer = player_lock
            .renderer_mut()
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();

        let actual_image = renderer.capture_frame().expect("Failed to capture image");

        let expected_image_path = base_path.join(format!("{name}.expected.png"));
        if expected_image_path.is_file() {
            let expected_image = image::open(&expected_image_path)
                .context("Failed to open expected image")?
                .into_rgba8();

            image_comparison.test(
                name,
                actual_image,
                expected_image,
                base_path,
                wgpu_descriptors.adapter.get_info(),
                known_failure,
            )?;
        } else if !known_failure {
            // If we're expecting this to be wrong, don't save a likely wrong image
            actual_image.save(expected_image_path)?;
        }
    } else if known_failure {
        // It's possible that the trace output matched but the image might not.
        // If we aren't checking the image, pretend the match failed (which makes it actually pass, since it's expecting failure).
        return Err(anyhow!(
            "Not checking images, pretending this failed since we don't know if it worked."
        ));
    }

    Ok(())
}
