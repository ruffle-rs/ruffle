use crate::set_logger;
use crate::util::options::TestOptions;
use crate::util::runner::{test_swf_approx, test_swf_with_hooks, RUN_IMG_TESTS};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Test {
    pub options: TestOptions,
    pub swf_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub name: String,
}

impl Test {
    pub fn from_options(options_path: &Path, root_dir: &Path) -> Result<Self> {
        let test_dir = options_path
            .parent()
            .context("Couldn't get test directory")?;
        let options = TestOptions::read(options_path).context("Couldn't load test options")?;
        let swf_path = test_dir.join("test.swf");
        let input_path = test_dir.join("input.json");
        let output_path = test_dir.join("output.txt");
        let name = test_dir
            .strip_prefix(root_dir)
            .context("Couldn't strip root prefix from test dir")?
            .to_string_lossy()
            .replace('\\', "/");
        Ok(Self {
            options,
            swf_path,
            input_path,
            output_path,
            name,
        })
    }

    pub fn run(self) -> Result<(), libtest_mimic::Failed> {
        set_logger();

        if let Some(approximations) = &self.options.approximations {
            test_swf_approx(
                &self.swf_path,
                self.options.num_frames,
                &self.input_path,
                &self.output_path,
                &approximations.number_patterns(),
                self.options.image,
                |actual, expected| approximations.compare(actual, expected),
            )
            .map_err(|e| e.to_string().into())
        } else {
            test_swf_with_hooks(
                &self.swf_path,
                self.options.num_frames,
                &self.input_path,
                &self.output_path,
                |player| {
                    if let Some(player_options) = &self.options.player_options {
                        player_options.setup(player);
                    }
                    Ok(())
                },
                |_| Ok(()),
                self.options.image,
                self.options.sleep_to_meet_frame_rate,
            )
            .map_err(|e| e.to_string().into())
        }
    }

    pub fn should_run(&self) -> bool {
        if self.options.ignore {
            return false;
        }
        if self.options.image && !RUN_IMG_TESTS {
            return false;
        }
        return true;
    }
}
