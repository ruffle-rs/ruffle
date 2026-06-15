# Fuzzing

Fuzzing (or fuzz testing) is a technique that feeds a program with large
numbers of automatically generated or mutated input, looking for inputs that
cause it to crash, hang, or otherwise misbehave (e.g. panics, out-of-bounds
reads, infinite loops, excessive memory use).
A fuzzer typically starts from a corpus of seed inputs, mutates them, and uses
code coverage feedback to guide the search towards new, interesting program
states.
Whenever a fuzzer finds an input that triggers a crash or hang, it saves that
input as an artifact so it can be reproduced, minimized, and turned into
a regression test.

## Why Ruffle needs fuzzing

Ruffle parses and executes SWF files (and other media), which are untrusted,
complex binary formats that can come from anywhere on the web.
By fuzzing, we can find and fix bugs caused by inputs that existing tests
don't cover: not just malformed files, but also unusual edge cases, and even
ordinary, real-world SWFs whose handling was simply never tested.

Example areas that benefit from fuzzing:

- **SWF and tag parsing** (the `swf` crate) — decompression, header parsing,
  and decoding of the many tag types into Ruffle's internal representation;

- **ActionScript bytecode** — decoding and interpretation of AVM1 and AVM2
  bytecode, including malformed or out-of-bounds constant pools, jumps, and
  opcodes;

- **Media decoders** — image (JPEG, PNG, GIF), video, and audio decoding,
  where malformed data is a common source of memory-safety bugs;

- **Regular expressions** — Ruffle's regex engine, used by ActionScript's
  `RegExp`, which can be exposed to untrusted patterns and input strings.

Ruffle's fuzz targets and tooling live under [`fuzz/`](../fuzz/README.md).

## Finding bugs with fuzz tests

To look for bugs, run a target for a while using the `cargo fuzzer` helper
from the repository root:

```sh
cargo fuzzer run parse_swf
```

If a crash or hang is found, `cargo fuzzer` reports the count and exits with a
non-zero status.

```
error: fuzzer found 1 crash(es) and 0 hang(s), check the artifacts directory
```

Artifacts are written under `fuzz/out/ruffle_fuzzer-<target>/<engine>/`.

Most of the time you don't need to run fuzzers, they will be run automatically
in CI.
