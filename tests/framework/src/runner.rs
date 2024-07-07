use crate::backends::{TestLogBackend, TestNavigatorBackend, TestUiBackend};
use crate::environment::RenderInterface;
use crate::fs_commands::{FsCommand, TestFsCommandProvider};
use crate::image_trigger::ImageTrigger;
use crate::options::{ImageComparison, TestOptions};
use crate::test::Test;
use crate::util::{read_bytes, write_image};
use anyhow::{anyhow, Result};
use image::ImageFormat;
use pretty_assertions::Comparison;
use ruffle_core::backend::navigator::NullExecutor;
use ruffle_core::events::{KeyCode, TextControlCode as RuffleTextControlCode};
use ruffle_core::events::{MouseButton as RuffleMouseButton, MouseWheelDelta};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{
    AutomatedEvent, InputInjector, MouseButton as InputMouseButton,
    TextControlCode as InputTextControlCode,
};
use ruffle_render::backend::{RenderBackend, ViewportDimensions};
use ruffle_socket_format::SocketEvent;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use vfs::VfsPath;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TestStatus {
    Continue,
    Sleep(Duration),
    Finished,
}

pub struct TestRunner {
    root_path: VfsPath,
    output_path: VfsPath,
    options: TestOptions,
    player: Arc<Mutex<Player>>,
    injector: InputInjector,
    executor: NullExecutor,
    frame_time: f64,
    frame_time_duration: Duration,
    log: TestLogBackend,
    fs_commands: mpsc::Receiver<FsCommand>,
    render_interface: Option<Box<dyn RenderInterface>>,
    images: HashMap<String, ImageComparison>,
    remaining_iterations: u32,
    current_iteration: u32,
}

impl TestRunner {
    pub fn new(
        test: &Test,
        movie: SwfMovie,
        injector: InputInjector,
        socket_events: Option<Vec<SocketEvent>>,
        renderer: Option<(Box<dyn RenderInterface>, Box<dyn RenderBackend>)>,
        viewport_dimensions: ViewportDimensions,
    ) -> Result<Self> {
        if test.options.num_frames.is_none() && test.options.num_ticks.is_none() {
            return Err(anyhow!(
                "Test {} must specify at least one of num_frames or num_ticks",
                &test.name
            ));
        }

        let executor = NullExecutor::new();
        let mut frame_time = 1000.0 / movie.frame_rate().to_f64();
        if let Some(tr) = test.options.tick_rate {
            frame_time = tr;
        }

        let frame_time_duration = Duration::from_millis(frame_time as u64);

        let log = TestLogBackend::default();
        let (fs_command_provider, fs_commands) = TestFsCommandProvider::new();
        let navigator = TestNavigatorBackend::new(
            test.root_path.clone(),
            &executor,
            socket_events,
            test.options.log_fetch.then(|| log.clone()),
        )?;

        let mut builder = PlayerBuilder::new()
            .with_log(log.clone())
            .with_navigator(navigator)
            .with_max_execution_duration(Duration::from_secs(300))
            .with_fs_commands(Box::new(fs_command_provider))
            .with_ui(TestUiBackend::new(test.fonts()?))
            .with_viewport_dimensions(
                viewport_dimensions.width,
                viewport_dimensions.height,
                viewport_dimensions.scale_factor,
            );

        let render_interface = if let Some((interface, backend)) = renderer {
            builder = builder.with_boxed_renderer(backend);
            Some(interface)
        } else {
            None
        };

        // Test player options may override anything set above
        let player = test
            .options
            .player_options
            .setup(builder)?
            .with_movie(movie)
            .with_autoplay(true) //.tick() requires playback
            .build();

        let images = test.options.image_comparisons.clone();

        let remaining_iterations = test
            .options
            .num_frames
            .or(test.options.num_ticks)
            .expect("valid iteration count");

        Ok(Self {
            root_path: test.root_path.clone(),
            output_path: test.output_path.clone(),
            player,
            injector,
            render_interface,
            executor,
            frame_time,
            frame_time_duration,
            log,
            fs_commands,
            images,
            remaining_iterations,
            current_iteration: 0,
            options: test.options.clone(),
        })
    }

    pub fn player(&self) -> &Arc<Mutex<Player>> {
        &self.player
    }

    pub fn options(&self) -> &TestOptions {
        &self.options
    }

    pub fn next_tick_may_be_last(&self) -> bool {
        self.remaining_iterations == 1
    }

    /// Tick this test forward, running any actionscript and progressing the timeline by one.
    pub fn tick(&mut self) {
        while !self
            .player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {}

        if self.options.num_ticks.is_some() {
            self.player.lock().unwrap().tick(self.frame_time);
        } else {
            self.player.lock().unwrap().run_frame();
            self.player.lock().unwrap().update_timers(self.frame_time);
            self.player.lock().unwrap().audio_mut().tick();
        }
        self.remaining_iterations -= 1;
        self.current_iteration += 1;
        self.executor.run();
    }

    /// After a tick, run any custom fdcommands that were queued up and perform any scheduled tests.
    pub fn test(&mut self) -> Result<TestStatus> {
        for command in self.fs_commands.try_iter() {
            match command {
                FsCommand::Quit => {
                    self.remaining_iterations = 0;
                }
                FsCommand::CaptureImage(name) => {
                    if let Some(image_comparison) = self.images.remove(&name) {
                        if image_comparison.trigger != ImageTrigger::FsCommand {
                            return Err(anyhow!("Encountered fscommand to capture and compare image '{name}', but the trigger was expected to be {:?}", image_comparison.trigger));
                        }
                        capture_and_compare_image(
                            &self.root_path,
                            &self.player,
                            &name,
                            image_comparison,
                            self.options.known_failure,
                            self.render_interface.as_deref(),
                        )?;
                    } else {
                        return Err(anyhow!("Encountered fscommand to capture and compare image '{name}', but no [image_comparison] was set up for this."));
                    }
                }
            }
        }

        self.injector.next(|evt, _btns_down| {
            let mut player = self.player.lock().unwrap();
            if let AutomatedEvent::SetClipboardText { text } = evt {
                player.ui_mut().set_clipboard_content(text.to_owned());
                return;
            }

            let handled = player.handle_event(match evt {
                AutomatedEvent::MouseDown {
                    pos, btn, index, ..
                } => PlayerEvent::MouseDown {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                    // None here means that the core will compute index automatically,
                    // however we do not want that in tests.
                    index: Some(index.unwrap_or_default()),
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
                AutomatedEvent::MouseWheel { lines, pixels } => PlayerEvent::MouseWheel {
                    delta: match (lines, pixels) {
                        (Some(lines), None) => MouseWheelDelta::Lines(*lines),
                        (None, Some(pixels)) => MouseWheelDelta::Pixels(*pixels),
                        _ => panic!("MouseWheel: expected only one of 'lines' or 'pixels'"),
                    },
                },
                AutomatedEvent::KeyDown { key_code } => PlayerEvent::KeyDown {
                    key_code: KeyCode::from_u8(*key_code).expect("Invalid keycode in test"),
                    key_char: None,
                },
                AutomatedEvent::KeyUp { key_code } => PlayerEvent::KeyUp {
                    key_code: KeyCode::from_u8(*key_code).expect("Invalid keycode in test"),
                    key_char: None,
                },
                AutomatedEvent::TextInput { codepoint } => PlayerEvent::TextInput {
                    codepoint: *codepoint,
                },
                AutomatedEvent::TextControl { code } => PlayerEvent::TextControl {
                    code: match code {
                        InputTextControlCode::MoveLeft => RuffleTextControlCode::MoveLeft,
                        InputTextControlCode::MoveLeftWord => RuffleTextControlCode::MoveLeftWord,
                        InputTextControlCode::MoveLeftLine => RuffleTextControlCode::MoveLeftLine,
                        InputTextControlCode::MoveLeftDocument => {
                            RuffleTextControlCode::MoveLeftDocument
                        }
                        InputTextControlCode::MoveRight => RuffleTextControlCode::MoveRight,
                        InputTextControlCode::MoveRightWord => RuffleTextControlCode::MoveRightWord,
                        InputTextControlCode::MoveRightLine => RuffleTextControlCode::MoveRightLine,
                        InputTextControlCode::MoveRightDocument => {
                            RuffleTextControlCode::MoveRightDocument
                        }
                        InputTextControlCode::SelectLeft => RuffleTextControlCode::SelectLeft,
                        InputTextControlCode::SelectLeftWord => {
                            RuffleTextControlCode::SelectLeftWord
                        }
                        InputTextControlCode::SelectLeftLine => {
                            RuffleTextControlCode::SelectLeftLine
                        }
                        InputTextControlCode::SelectLeftDocument => {
                            RuffleTextControlCode::SelectLeftDocument
                        }
                        InputTextControlCode::SelectRight => RuffleTextControlCode::SelectRight,
                        InputTextControlCode::SelectRightWord => {
                            RuffleTextControlCode::SelectRightWord
                        }
                        InputTextControlCode::SelectRightLine => {
                            RuffleTextControlCode::SelectRightLine
                        }
                        InputTextControlCode::SelectRightDocument => {
                            RuffleTextControlCode::SelectRightDocument
                        }
                        InputTextControlCode::SelectAll => RuffleTextControlCode::SelectAll,
                        InputTextControlCode::Copy => RuffleTextControlCode::Copy,
                        InputTextControlCode::Paste => RuffleTextControlCode::Paste,
                        InputTextControlCode::Cut => RuffleTextControlCode::Cut,
                        InputTextControlCode::Backspace => RuffleTextControlCode::Backspace,
                        InputTextControlCode::Enter => RuffleTextControlCode::Enter,
                        InputTextControlCode::Delete => RuffleTextControlCode::Delete,
                    },
                },
                AutomatedEvent::FocusGained => PlayerEvent::FocusGained,
                AutomatedEvent::FocusLost => PlayerEvent::FocusLost,
                AutomatedEvent::Wait | AutomatedEvent::SetClipboardText { .. } => unreachable!(),
            });

            #[allow(clippy::single_match)]
            match evt {
                AutomatedEvent::MouseDown {
                    assert_handled: Some(assert_handled),
                    ..
                } => {
                    if handled != assert_handled.value {
                        panic!(
                            "Event handled status assertion failed: \n\
                            \x20   expected to be handled: {}\n\
                            \x20   was handled: {}\n\
                            \x20   message: {}",
                            assert_handled.value, handled, assert_handled.message
                        );
                    }
                }
                _ => {}
            }
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        self.player.lock().unwrap().render();

        if let Some(name) = self
            .images
            .iter()
            .find(|(_k, v)| v.trigger == ImageTrigger::SpecificIteration(self.current_iteration))
            .map(|(k, _v)| k.to_owned())
        {
            let image_comparison = self
                .images
                .remove(&name)
                .expect("Name was just retrieved from map, should not be missing!");
            capture_and_compare_image(
                &self.root_path,
                &self.player,
                &name,
                image_comparison,
                self.options.known_failure,
                self.render_interface.as_deref(),
            )?;
        }

        if self.remaining_iterations == 0 {
            // Last iteration, let's check everything went well

            if let Some(name) = self
                .images
                .iter()
                .find(|(_k, v)| v.trigger == ImageTrigger::LastFrame)
                .map(|(k, _v)| k.to_owned())
            {
                let image_comparison = self
                    .images
                    .remove(&name)
                    .expect("Name was just retrieved from map, should not be missing!");

                capture_and_compare_image(
                    &self.root_path,
                    &self.player,
                    &name,
                    image_comparison,
                    self.options.known_failure,
                    self.render_interface.as_deref(),
                )?;
            }

            if !self.images.is_empty() {
                return Err(anyhow!(
                    "Image comparisons didn't trigger: {:?}",
                    self.images.keys()
                ));
            }

            self.executor.run();

            let trace = self.log.trace_output();
            // Null bytes are invisible, and interfere with constructing
            // the expected output.txt file. Any tests dealing with null
            // bytes should explicitly test for them in ActionScript.
            let normalized_trace = trace.replace('\0', "");
            self.compare_output(&normalized_trace)?;
        }

        Ok(match self.remaining_iterations {
            0 => TestStatus::Finished,
            _ if self.options.sleep_to_meet_frame_rate => {
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
                TestStatus::Sleep(self.frame_time_duration)
            }
            _ => TestStatus::Continue,
        })
    }

    pub fn compare_output(&self, actual_output: &str) -> Result<()> {
        let expected_output = self.output_path.read_to_string()?.replace("\r\n", "\n");

        if let Some(approximations) = &self.options.approximations {
            if actual_output.lines().count() != expected_output.lines().count() {
                return Err(anyhow!(
                    "# of lines of output didn't match (expected {} from Flash, got {} from Ruffle",
                    expected_output.lines().count(),
                    actual_output.lines().count()
                ));
            }

            for (actual, expected) in actual_output.lines().zip(expected_output.lines()) {
                // If these are numbers, compare using approx_eq.
                if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>())
                {
                    // NaNs should be able to pass in an approx test.
                    if actual.is_nan() && expected.is_nan() {
                        continue;
                    }

                    approximations.compare(actual, expected)?;
                } else {
                    let mut found = false;

                    // Check each of the user-provided regexes for a match
                    for pattern in approximations.number_patterns() {
                        if let (Some(actual_captures), Some(expected_captures)) =
                            (pattern.captures(actual), pattern.captures(expected))
                        {
                            found = true;
                            if expected_captures.len() != actual_captures.len() {
                                return Err(anyhow!(
                                    "Differing numbers of regex captures (expected {}, actually {})",
                                    expected_captures.len(),
                                    actual_captures.len(),
                                ));
                            }

                            // Each capture group (other than group 0, which is always the entire regex
                            // match) represents a floating-point value
                            for (actual_val, expected_val) in actual_captures
                                .iter()
                                .skip(1)
                                .zip(expected_captures.iter().skip(1))
                            {
                                let actual_num = actual_val
                                    .expect("Missing capture group value for 'actual'")
                                    .as_str()
                                    .parse::<f64>()
                                    .expect("Failed to parse 'actual' capture group as float");
                                let expected_num = expected_val
                                    .expect("Missing capture group value for 'expected'")
                                    .as_str()
                                    .parse::<f64>()
                                    .expect("Failed to parse 'expected' capture group as float");
                                approximations.compare(actual_num, expected_num)?;
                            }
                            let modified_actual = pattern.replace_all(actual, "");
                            let modified_expected = pattern.replace_all(expected, "");

                            assert_text_matches(
                                modified_actual.as_ref(),
                                modified_expected.as_ref(),
                            )?;
                            break;
                        }
                    }

                    if !found {
                        assert_text_matches(actual, expected)?;
                    }
                }
            }
        } else {
            assert_text_matches(actual_output, &expected_output)?;
        }

        Ok(())
    }
}

fn capture_and_compare_image(
    base_path: &VfsPath,
    player: &Arc<Mutex<Player>>,
    name: &String,
    image_comparison: ImageComparison,
    known_failure: bool,
    render_interface: Option<&dyn RenderInterface>,
) -> Result<()> {
    use anyhow::Context;

    if let Some(render_interface) = render_interface {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();

        let actual_image = render_interface.capture(player_lock.renderer_mut());

        let expected_image_path = base_path.join(format!("{name}.expected.png"))?;
        if expected_image_path.is_file()? {
            let expected_image = image::load_from_memory(&read_bytes(&expected_image_path)?)
                .context("Failed to open expected image")?
                .into_rgba8();

            image_comparison.test(
                name,
                actual_image,
                expected_image,
                base_path,
                render_interface.name(),
                known_failure,
            )?;
        } else if known_failure {
            return Err(anyhow!(
                "No image to compare to, pretending this failed since we don't know if it worked."
            ));
        } else {
            // If we're expecting this to be wrong, don't save a likely wrong image
            write_image(&expected_image_path, &actual_image, ImageFormat::Png)?;
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

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

fn assert_text_matches(ruffle: &str, flash: &str) -> Result<()> {
    if flash != ruffle {
        let left_pretty = PrettyString(ruffle);
        let right_pretty = PrettyString(flash);
        let comparison = Comparison::new(&left_pretty, &right_pretty);

        Err(anyhow!(
            "assertion failed: `(flash_expected == ruffle_actual)`\
                       \n\
                       \n{}\
                       \n",
            comparison
        ))
    } else {
        Ok(())
    }
}
