use crate::flashplayer::FlashPlayer;
use crate::flashplayer::config::FlashHarnessConfig;
use crate::flashplayer::environment::PlayerEnvironment;
use crate::{TestFilterOptions, find_root_dir, load_test_dir};
use clap::Args;
use ruffle_core::DEFAULT_PLAYER_VERSION;
use ruffle_fs_tests_runner::FsTestsRunner;
use std::path::Path;
use std::time::Duration;

#[derive(Args)]
pub struct ExecuteOptions {
    #[command(flatten)]
    filter: TestFilterOptions,

    /// Forcefully quit the test after the given amount of seconds has elapsed.
    #[arg(long, default_value_t = 3.0)]
    max_duration: f32,

    /// Forcefully quit the test after the given amount of seconds of inactivity (no log output).
    #[arg(long, default_value_t = 0.5)]
    max_idle: f32,
}

pub fn main_execute(options: ExecuteOptions) {
    if options.filter.is_empty() {
        panic!(
            "Running `execute` on every single test is inadvisable. Please specify any kind of filter."
        );
    }

    let config = FlashHarnessConfig::load().unwrap();
    let mut runner = FsTestsRunner::new();
    options.filter.apply(&mut runner);
    runner
        .with_root_dir(find_root_dir())
        .with_canonicalize_paths(true)
        .with_test_loader(Box::new(move |params, register_trial| {
            for test in load_test_dir(&params.test_dir, params.test_name) {
                register_trial((params.test_dir_real.clone().into_owned(), test));
            }
        }))
        .with_sorter(Box::new(|a, b| a.1.name().cmp(b.1.name())));
    let tests = runner.find_tests();
    for (test_dir, test) in tests {
        let swf_path_real = test_dir.join(test.swf_path.as_str().trim_start_matches(['/']));
        let swf_path_real_str = swf_path_real.to_string_lossy();
        let environment = PlayerEnvironment::new();
        let version = test
            .options
            .player_options
            .version()
            .unwrap_or(DEFAULT_PLAYER_VERSION);
        let definition = config.get_player(version).unwrap_or_else(|| {
            panic!(
                "Could not find a Flash Player debug executable for version {} in .flash_players.toml",
                version
            )
        });
        let player = FlashPlayer::new(definition);
        let output = player.run(
            &environment,
            Path::new(swf_path_real_str.as_ref()),
            Duration::from_secs_f32(options.max_duration),
            Duration::from_secs_f32(options.max_idle),
        );
        println!("Test output:\n{}", output);
    }
}
