use std::borrow::Cow;

use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::{FrameSelection, Opt};

pub struct ExporterProgress {
    progress: Option<ProgressBar>,
}

impl ExporterProgress {
    pub fn new(opt: &Opt, files_count: u64) -> Self {
        let progress = if !opt.silent {
            let progress = match opt.frames {
                FrameSelection::Count(n) => ProgressBar::new(files_count * (n.get() as u64)),
                _ => ProgressBar::new_spinner(), // TODO Once we figure out a way to get framecount before calling take_screenshot, then this can be changed back to a progress bar when using --frames all
            };
            progress.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] {pos:>7}/{len:7} {msg}",
                )
                .unwrap()
                .progress_chars("##-"),
            );
            Some(progress)
        } else {
            None
        };
        Self { progress }
    }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(progress) = &self.progress {
            progress.set_message(msg);
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(progress) = &self.progress {
            progress.inc(delta);
        }
    }

    pub fn finish_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(progress) = &self.progress {
            progress.finish_with_message(msg);
        } else {
            println!("{}", msg.into());
        }
    }
}
