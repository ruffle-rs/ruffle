use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use ruffle_fuzzer::{
    Fuzzer, build_fuzz_targets, list_targets, prepare_swf_tests_corpus, run_fuzz_targets,
};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Parser)]
#[command(about = "Ruffle fuzzing helper")]
#[command(long_about = "\
Ruffle fuzzing helper.

Wraps cargo-fuzz (libFuzzer) and cargo-afl (AFL++) to prepare corpora, build,
and run Ruffle's fuzz targets.

Example usage:

    cargo fuzzer run parse_swf --time 5m --fuzzer afl --fuzzer libfuzzer

Use --fuzzer to restrict to a specific engine (AFL++ or libFuzzer); libFuzzer
requires a nightly Rust toolchain and cargo-fuzz, AFL++ requires cargo-afl.
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available fuzz targets.
    List {
        /// Long format. Lists additional information.
        #[arg(long, short, default_value_t = false)]
        long: bool,

        /// List of fuzzers to list targets for. By default lists targets for
        /// all fuzzers.
        #[arg(long, value_enum)]
        fuzzer: Vec<Fuzzer>,
    },

    /// Prepare fuzzing corpus.
    Prepare,

    /// Build fuzz targets.
    Build {
        /// Target name. Builds all targets if empty.
        target: Option<String>,

        /// List of fuzzers to build for. By default builds for all fuzzers.
        #[arg(long, value_enum)]
        fuzzer: Vec<Fuzzer>,
    },

    /// Prepare fuzzing corpus, build, and run fuzz targets.
    Run {
        /// Target name. Runs all targets if empty.
        target: Option<String>,

        /// Do not prepare.
        #[arg(long, default_value_t = false)]
        no_prepare: bool,

        /// Do not build.
        #[arg(long, default_value_t = false)]
        no_build: bool,

        /// Stop fuzzing after the given time (e.g. 30s, 5m, 2h).
        #[arg(long, short, value_parser = humantime::parse_duration)]
        time: Option<Duration>,

        /// List of fuzzers to run. By default runs all fuzzers.
        #[arg(long, value_enum)]
        fuzzer: Vec<Fuzzer>,
    },
}

fn main() {
    let matches = Cli::command().bin_name("cargo fuzzer").get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    let all = list_targets();

    match cli.command {
        Commands::List { long, fuzzer } => {
            let fuzzer_filter: HashSet<Fuzzer> = fuzzer.into_iter().collect();

            for (name, target) in &all {
                let target_fuzzers = &target.supported_fuzzers;

                if !fuzzer_filter.intersection(target_fuzzers).any(|_| true) {
                    continue;
                }

                if long {
                    let names: Vec<_> = target_fuzzers.iter().map(|f| f.to_string()).collect();
                    println!("{name}  [{}]", names.join(", "));
                } else {
                    println!("{name}");
                }
            }
        }
        Commands::Prepare => {
            prepare_swf_tests_corpus();
        }
        Commands::Build { target, fuzzer } => {
            build_fuzz_targets(&all, target.as_deref(), &fuzzer);
        }
        Commands::Run {
            target,
            no_prepare,
            no_build,
            time,
            fuzzer,
        } => {
            if !no_prepare {
                prepare_swf_tests_corpus();
            }

            if !no_build {
                build_fuzz_targets(&all, target.as_deref(), &fuzzer);
            }

            let stats = run_fuzz_targets(&all, target.as_deref(), &fuzzer, time);
            if !stats.success() {
                eprintln!(
                    "error: fuzzer found {} crash(es) and {} hang(s), check the artifacts directory",
                    stats.crashes, stats.hangs
                );
                std::process::exit(1);
            }
        }
    }
}
