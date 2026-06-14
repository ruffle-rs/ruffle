use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum Fuzzer {
    Afl,
    Libfuzzer,
}

impl Fuzzer {
    pub fn cargo_tool_name(self) -> &'static str {
        match self {
            Fuzzer::Afl => "cargo-afl",
            Fuzzer::Libfuzzer => "cargo-fuzz",
        }
    }

    pub fn cargo_tool_command(self) -> &'static str {
        match self {
            Fuzzer::Afl => "afl",
            Fuzzer::Libfuzzer => "fuzz",
        }
    }
}

impl std::fmt::Display for Fuzzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fuzzer::Afl => write!(f, "AFL++"),
            Fuzzer::Libfuzzer => write!(f, "libFuzzer"),
        }
    }
}

pub struct Target {
    pub name: String,
    pub supported_fuzzers: HashSet<Fuzzer>,
}

pub type Targets = BTreeMap<String, Target>;

pub struct RunStats {
    pub crashes: usize,
    pub hangs: usize,
}

impl RunStats {
    pub fn success(&self) -> bool {
        self.crashes == 0 && self.hangs == 0
    }
}

impl std::iter::Sum for RunStats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            RunStats {
                crashes: 0,
                hangs: 0,
            },
            |a, b| RunStats {
                crashes: a.crashes + b.crashes,
                hangs: a.hangs + b.hangs,
            },
        )
    }
}

pub fn fuzz_dir() -> PathBuf {
    let dir = std::env::var("LOCAL_RUFFLE_FUZZ_WORKSPACE_DIR")
        .expect("LOCAL_RUFFLE_FUZZ_WORKSPACE_DIR is not defined");
    PathBuf::from(dir)
}

pub fn swf_tests_dir() -> PathBuf {
    let dir = std::env::var("LOCAL_RUFFLE_TESTS_SWFS_DIR")
        .expect("LOCAL_RUFFLE_TESTS_SWFS_DIR is not defined");
    PathBuf::from(dir)
}

pub fn prepare_swf_tests_corpus() {
    let swf_tests_dir = swf_tests_dir();
    let dest = fuzz_dir().join("corpus").join("swf").join("swf_tests");

    if dest.exists() {
        fs::remove_dir_all(&dest).unwrap();
    }

    fs::create_dir_all(&dest).unwrap();

    for entry in walkdir::WalkDir::new(&swf_tests_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "swf"))
    {
        let relative = entry.path().strip_prefix(&swf_tests_dir).unwrap();
        let flat_name = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("_");
        fs::copy(entry.path(), dest.join(flat_name)).unwrap();
    }
}

fn corpus_for(target: &str) -> PathBuf {
    let corpuses_path = fuzz_dir().join("corpuses.toml");
    let table: toml::Table = fs::read_to_string(&corpuses_path)
        .unwrap_or_else(|e| panic!("Failed to read corpuses.toml: {e}"))
        .parse()
        .unwrap_or_else(|e| panic!("corpuses.toml failed to parse: {e}"));
    let subdir = table
        .get(target)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("Missing corpus for {target}"));
    fuzz_dir().join("corpus").join(subdir)
}

fn list_fuzz_targets_in(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| {
                    let name = e.ok()?.file_name().into_string().ok()?;
                    name.strip_suffix(".rs").map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn list_targets_afl() -> BTreeSet<String> {
    list_fuzz_targets_in(&fuzz_dir().join("fuzz_targets_afl"))
}

fn list_targets_libfuzzer() -> BTreeSet<String> {
    list_fuzz_targets_in(&fuzz_dir().join("fuzz_targets"))
}

pub fn list_targets() -> Targets {
    let afl = list_targets_afl();
    let lf = list_targets_libfuzzer();
    afl.union(&lf)
        .cloned()
        .map(|name| {
            let mut supported_fuzzers = HashSet::new();
            if afl.contains(&name) {
                supported_fuzzers.insert(Fuzzer::Afl);
            }
            if lf.contains(&name) {
                supported_fuzzers.insert(Fuzzer::Libfuzzer);
            }
            let target = Target {
                name: name.clone(),
                supported_fuzzers,
            };
            (name, target)
        })
        .collect()
}

/// Check the selected fuzzers, and report any issues related to their setup.
pub fn check_fuzzers(fuzzer: &[Fuzzer]) -> Result<(), anyhow::Error> {
    if fuzzer.is_empty() || fuzzer.contains(&Fuzzer::Libfuzzer) {
        check_fuzzer(Fuzzer::Libfuzzer)?;
    }

    if fuzzer.is_empty() || fuzzer.contains(&Fuzzer::Afl) {
        check_fuzzer(Fuzzer::Afl)?;
    }

    Ok(())
}

fn check_fuzzer(fuzzer: Fuzzer) -> Result<(), anyhow::Error> {
    let different_engine_message = "Alternatively, you can use --fuzzer \
        to select a different fuzzing engine.";

    if fuzzer == Fuzzer::Libfuzzer {
        let version = Command::new("rustc")
            .args(["--version"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()?
            .stdout;
        let version = String::from_utf8_lossy(&version);
        if !version.contains("nightly") {
            return Err(anyhow!(
                "{fuzzer} requires a nightly Rust toolchain, but the active \
                toolchain is not nightly:\n\n    {}\n\n\
                {different_engine_message}",
                version.trim()
            ));
        }
    }

    let available = Command::new("cargo")
        .args([fuzzer.cargo_tool_command(), "--help"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success();
    if !available {
        return Err(anyhow!(
            "In order to use {fuzzer}, {} needs to be installed:\n\n    \
                cargo install {}\n\n\
            {different_engine_message}",
            fuzzer.cargo_tool_name(),
            fuzzer.cargo_tool_name()
        ));
    }

    Ok(())
}

pub fn build_fuzz_targets(all: &Targets, target: Option<&str>, fuzzer: &[Fuzzer]) {
    let names: Vec<String> = match target {
        Some(name) => {
            assert!(all.contains_key(name), "target '{name}' not found");
            vec![name.to_string()]
        }
        None => all.keys().cloned().collect(),
    };

    let build_afl = fuzzer.is_empty() || fuzzer.contains(&Fuzzer::Afl);
    let build_lf = fuzzer.is_empty() || fuzzer.contains(&Fuzzer::Libfuzzer);

    let mut afl_bins: Vec<String> = vec![];
    let mut lf_targets: Vec<String> = vec![];

    for name in names {
        let supported = &all[&name].supported_fuzzers;
        if build_afl && supported.contains(&Fuzzer::Afl) {
            afl_bins.push(format!("{name}_afl"));
        }
        if build_lf && supported.contains(&Fuzzer::Libfuzzer) {
            lf_targets.push(name);
        }
    }

    let fuzz_dir = fuzz_dir();

    if !afl_bins.is_empty() {
        let mut cmd = Command::new("cargo");
        cmd.args(["afl", "build", "--no-default-features", "--features", "afl"]);
        for bin in &afl_bins {
            cmd.args(["--bin", bin]);
        }
        let status = cmd
            .current_dir(&fuzz_dir)
            .status()
            .expect("afl build failed");
        assert!(status.success(), "afl build failed");
    }

    for target in lf_targets {
        let status = Command::new("cargo")
            .args(["fuzz", "build", &target])
            .current_dir(&fuzz_dir)
            .status()
            .expect("cargo fuzz build failed");
        assert!(status.success(), "cargo fuzz build failed: {target}");
    }
}

pub fn run_fuzz_targets(
    all: &Targets,
    target: Option<&str>,
    fuzzer: &[Fuzzer],
    time: Option<Duration>,
) -> RunStats {
    let fuzz_dir = fuzz_dir();

    let names: Vec<String> = match target {
        Some(name) => {
            assert!(all.contains_key(name), "target '{name}' not found");
            vec![name.to_string()]
        }
        None => all.keys().cloned().collect(),
    };

    let single_process = names.len() == 1 && fuzzer.len() == 1;

    let mut children: Vec<(Fuzzer, PathBuf, std::process::Child)> = vec![];

    for name in &names {
        let supported = &all[name].supported_fuzzers;

        let corpus = corpus_for(name);

        for f in [Fuzzer::Afl, Fuzzer::Libfuzzer] {
            if !fuzzer.is_empty() && !fuzzer.contains(&f) {
                // Filtered out.
                continue;
            }
            if !supported.contains(&f) {
                // Not supported by target.
                continue;
            }

            let out_dir = fuzz_dir.join("out").join(format!("ruffle_fuzzer-{name}"));
            children.push((
                f,
                out_dir.clone(),
                spawn_fuzz_process(f, name, &corpus, &fuzz_dir, &out_dir, single_process, time),
            ));
        }
    }

    children
        .into_iter()
        .map(|(f, out_dir, mut child)| {
            child.wait().ok();
            match f {
                Fuzzer::Afl => afl_stats(&out_dir.join("afl")),
                Fuzzer::Libfuzzer => libfuzzer_stats(&out_dir.join("libfuzzer")),
            }
        })
        .sum()
}

fn spawn_fuzz_process(
    fuzzer: Fuzzer,
    name: &str,
    corpus: &Path,
    fuzz_dir: &Path,
    out_dir: &Path,
    single_process: bool,
    time: Option<Duration>,
) -> std::process::Child {
    let mut cmd = Command::new("cargo");

    let log_dir = out_dir.join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let log_name = match fuzzer {
        Fuzzer::Afl => {
            let bin = format!("{name}_afl");
            let bin_path = format!("target/debug/{bin}");
            let out = out_dir.join("afl");
            fs::create_dir_all(&out).unwrap();
            cmd.args([
                "afl",
                "fuzz",
                "-i",
                corpus.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ]);
            if let Some(t) = time {
                cmd.args(["-V", &t.as_secs().to_string()]);
            }
            cmd.args(["--", &bin_path]);
            format!("{bin}.log")
        }
        Fuzzer::Libfuzzer => {
            let bin = format!("{name}_libfuzzer");
            let out = out_dir.join("libfuzzer");
            fs::create_dir_all(&out).unwrap();
            cmd.args([
                "fuzz",
                "run",
                name,
                corpus.to_str().unwrap(),
                "--",
                "-fork=1",
                // The following makes libfuzzer continue on crashes/ooms/timeouts.
                "-ignore_crashes=1",
                "-ignore_ooms=1",
                "-ignore_timeouts=1",
                &format!("-artifact_prefix={}/", out.to_str().unwrap()),
            ]);
            if let Some(t) = time {
                cmd.arg(format!("-max_total_time={}", t.as_secs()));
            }
            format!("{bin}.log")
        }
    };

    if single_process {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        let log_file = fs::File::create(log_dir.join(log_name)).expect("failed to create log file");
        cmd.stdout(log_file.try_clone().unwrap()).stderr(log_file);
    }

    cmd.current_dir(fuzz_dir)
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn {fuzzer} for {name}: {e}"))
}

fn afl_stats(out_dir: &Path) -> RunStats {
    let mut crashes = 0;
    let mut hangs = 0;
    for entry in walkdir::WalkDir::new(out_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        match entry.file_name().to_str() {
            Some("crashes") => crashes += afl_dir_count(entry.path()),
            Some("hangs") => hangs += afl_dir_count(entry.path()),
            _ => {}
        }
    }
    RunStats { crashes, hangs }
}

fn libfuzzer_stats(artifacts_dir: &Path) -> RunStats {
    let mut crashes = 0;
    let mut hangs = 0;
    if let Ok(entries) = fs::read_dir(artifacts_dir) {
        for name in entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .filter_map(|n| n.into_string().ok())
        {
            if name.starts_with("crash-") || name.starts_with("oom-") {
                crashes += 1;
            } else if name.starts_with("timeout-") {
                hangs += 1;
            }
        }
    }
    RunStats { crashes, hangs }
}

fn afl_dir_count(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|e| {
            e.filter(|e| e.as_ref().is_ok_and(|e| e.file_name() != "README.txt"))
                .count()
        })
        .unwrap_or(0)
}
