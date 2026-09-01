//! Periodic memory sampling, used to measure SWF retention over a play session.
//!
//! Enabled with `--memory-report <FILE>`. Every interval it writes one CSV row
//! combining the process' resident set size with Ruffle's own accounting of
//! what each still-loaded movie is keeping alive, so that growth in RSS can be
//! attributed to (or ruled out as) movies that were supposed to be unloaded.

use ruffle_core::Player;
use ruffle_core::memory_report::MemoryReport;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct MemoryReporter {
    output: BufWriter<File>,
    interval: Duration,
    started: Instant,
    last_sample: Option<Instant>,
    /// Retained bytes at the first sample, so the log states growth directly.
    baseline_retained: Option<usize>,
}

impl MemoryReporter {
    pub fn new(path: &Path, interval: Duration) -> Result<Self, std::io::Error> {
        let mut output = BufWriter::new(File::create(path)?);
        writeln!(output, "rss_bytes,{}", MemoryReport::csv_header())?;
        output.flush()?;

        Ok(Self {
            output,
            interval,
            started: Instant::now(),
            last_sample: None,
            baseline_retained: None,
        })
    }

    /// Takes a sample if the interval has elapsed. Cheap to call every frame.
    pub fn maybe_sample(&mut self, player: &mut Player) {
        let now = Instant::now();
        if let Some(last) = self.last_sample
            && now.duration_since(last) < self.interval
        {
            return;
        }
        self.last_sample = Some(now);

        let report = player.mutate_with_update_context(MemoryReport::capture);
        let elapsed = now.duration_since(self.started).as_secs_f64();
        let rss = resident_set_size().unwrap_or(0);

        if let Err(e) = writeln!(self.output, "{},{}", rss, report.to_csv_row(elapsed))
            .and_then(|_| self.output.flush())
        {
            tracing::error!("Could not write memory report: {e}");
        }

        let retained = report.swf_bytes + report.bitmap_decoded_bytes;
        let baseline = *self.baseline_retained.get_or_insert(retained);

        tracing::info!(
            "memory @{elapsed:.0}s: rss {} MiB, {} movies retaining {} MiB \
             (+{} MiB since first sample), {} pending loaders, {} class aliases{}",
            rss / (1024 * 1024),
            report.movies.len(),
            retained / (1024 * 1024),
            retained.saturating_sub(baseline) / (1024 * 1024),
            report.pending_loaders,
            report.class_aliases,
            report.top_movies(5),
        );
    }
}

/// Resident set size of this process, in bytes.
#[cfg(target_os = "linux")]
fn resident_set_size() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    let kb: usize = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

#[cfg(not(target_os = "linux"))]
fn resident_set_size() -> Option<usize> {
    None
}
