mod automation;
mod image_test;
mod trace;

use crate::backends::{TestLogBackend, TestNavigatorBackend, TestUiBackend};
use crate::environment::RenderInterface;
use crate::fs_commands::{FsCommand, TestFsCommandProvider};
use crate::image_trigger::ImageTrigger;
use crate::options::TestOptions;
use crate::options::image_comparison::ImageComparison;
use crate::options::known_failure::KnownFailure;
use crate::runner::automation::perform_automated_event;
use crate::runner::image_test::capture_and_compare_image;
use crate::runner::trace::compare_trace_output;
use crate::test::Test;
use anyhow::{Result, anyhow};
use ruffle_core::backend::navigator::NullExecutor;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_input_format::InputInjector;
use ruffle_render::backend::{RenderBackend, ViewportDimensions};
use ruffle_socket_format::SocketEvent;
use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
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
    preloaded: bool,
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
            .with_ui(TestUiBackend::new(test.fonts()?, test.font_sorts()))
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

        test.options
            .default_fonts
            .apply(&mut player.lock().unwrap());

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
            preloaded: false,
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

    pub fn is_preloaded(&self) -> bool {
        self.preloaded
    }

    /// Ticks this test forward: runs actionscript, progresses the timeline by one,
    /// executes custom FsCommands and performs scheduled tests.
    pub fn tick(&mut self) -> Result<TestStatus> {
        use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

        let unwind_result = catch_unwind(AssertUnwindSafe(|| self.do_tick()));
        match (unwind_result, &self.options.known_failure) {
            (Ok(()), _) => (),
            (Err(panic), KnownFailure::Panic { message }) => {
                let actual = panic_payload_as_string(panic);
                if actual.contains(message) {
                    return Ok(TestStatus::Finished);
                }

                let mut actual = actual.into_owned();
                actual.push_str("\n\nnote: expected panic message to contain: ");
                actual.push_str(message);
                resume_unwind(Box::new(actual))
            }
            (Err(panic), _) => resume_unwind(panic),
        }

        self.test()?;

        match self.remaining_iterations {
            0 => self.last_test().map(|_| TestStatus::Finished),
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
                Ok(TestStatus::Sleep(self.frame_time_duration))
            }
            _ => Ok(TestStatus::Continue),
        }
    }

    fn do_tick(&mut self) {
        if !self
            .player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {
            self.executor.run();
            return;
        }
        self.preloaded = true;

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
    fn test(&mut self) -> Result<()> {
        if !self.preloaded {
            return Ok(());
        }
        for command in self.fs_commands.try_iter() {
            match command {
                FsCommand::Quit => {
                    self.remaining_iterations = 0;
                }
                FsCommand::CaptureImage(name) => {
                    if let Some(image_comparison) = self.images.remove(&name) {
                        if image_comparison.trigger != ImageTrigger::FsCommand {
                            return Err(anyhow!(
                                "Encountered fscommand to capture and compare image '{name}', but the trigger was expected to be {:?}",
                                image_comparison.trigger
                            ));
                        }
                        capture_and_compare_image(
                            &self.root_path,
                            &self.player,
                            &name,
                            image_comparison,
                            self.render_interface.as_deref(),
                        )?;
                    } else {
                        return Err(anyhow!(
                            "Encountered fscommand to capture and compare image '{name}', but no [image_comparison] was set up for this."
                        ));
                    }
                }
            }
        }

        self.injector.next(|evt, _btns_down| {
            let mut player = self.player.lock().unwrap();
            perform_automated_event(evt, &mut player);
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        self.player.lock().unwrap().render();

        let trigger = ImageTrigger::SpecificIteration(self.current_iteration);
        if let Some((name, comp)) = self.take_image_comparison_by_trigger(trigger) {
            capture_and_compare_image(
                &self.root_path,
                &self.player,
                &name,
                comp,
                self.render_interface.as_deref(),
            )?;
        }

        Ok(())
    }

    fn last_test(&mut self) -> Result<()> {
        // Last iteration, let's check everything went well
        if let KnownFailure::Panic { .. } = &self.options.known_failure {
            return Err(anyhow!(
                "Test was known to be panicking, but now finishes successfully. \
                Please update it and remove `known_failure.panic = '...'`!",
            ));
        }

        let trigger = ImageTrigger::LastFrame;
        if let Some((name, comp)) = self.take_image_comparison_by_trigger(trigger) {
            capture_and_compare_image(
                &self.root_path,
                &self.player,
                &name,
                comp,
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

        compare_trace_output(
            &self.log,
            &self.output_path,
            self.options.approximations.as_ref(),
            &self.options.known_failure,
        )
    }

    fn take_image_comparison_by_trigger(
        &mut self,
        trigger: ImageTrigger,
    ) -> Option<(String, ImageComparison)> {
        self.images.extract_if(|_k, v| v.trigger == trigger).next()
    }
}

fn panic_payload_as_string(panic: Box<dyn Any + Send + 'static>) -> Cow<'static, str> {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).into()
    } else if let Ok(s) = panic.downcast::<String>() {
        (*s).into()
    } else {
        "<opaque payload>".into()
    }
}
