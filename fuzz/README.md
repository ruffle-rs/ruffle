# Ruffle Fuzzing

This directory contains fuzz targets for Ruffle, set up as a separate Cargo
workspace (so it doesn't pollute the main workspace's lock file).

Targets support two fuzzing engines:

- [libFuzzer](https://llvm.org/docs/LibFuzzer.html) via
  [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (requires a nightly
  Rust toolchain)
- [AFL++](https://aflplus.plus/) via
  [`cargo-afl`](https://github.com/rust-fuzz/afl.rs)

## Usage

The easiest way to interact with the fuzz targets is through the `cargo
fuzzer` alias (defined in `.cargo/config.toml`, backed by `tools/fuzzer`),
run from the repository root:

```sh
cargo fuzzer --help
```

## Layout

- `fuzz_targets/` — libFuzzer entry points (one `.rs` file per target)
- `fuzz_targets_afl/` — AFL++ entry points for the same targets
- `src/` — the actual fuzzing logic shared by both engines
- `corpus/` — seed inputs used by the fuzzers, organized per `corpuses.toml`
- `corpuses.toml` — maps each target to its corpus subdirectory
- `out/`, `artifacts/`, `target/` — generated output (git-ignored)

## Adding a new target

1. Add the fuzzing logic as a function in `src/`, exported from `src/lib.rs`.
2. Add an entry point in `fuzz_targets/<name>.rs` (libFuzzer) and/or
   `fuzz_targets_afl/<name>.rs` (AFL++) that calls it.
3. Register the binary in `Cargo.toml` under `[[bin]]`.
4. Add a corpus entry for the target in `corpuses.toml`.
