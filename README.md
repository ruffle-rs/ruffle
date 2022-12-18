<p align="center">
  <a href="https://ruffle.rs"><img alt="Ruffle" src="https://ruffle.rs/assets/logo.svg" /></a>
</p>
<p align="center">
  <a href="https://github.com/ruffle-rs/ruffle/actions">
    <img alt="Rust Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_rust.yml?label=Rust%20Build&logo=github&branch=master" />
    <img alt="Web Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_web.yml?label=Web%20Build&logo=github&branch=master" />
  </a>
  <a href="https://www.npmjs.com/package/@ruffle-rs/ruffle">
    <img alt="Ruffle npm" src="https://img.shields.io/npm/v/@ruffle-rs/ruffle?color=007acc&logo=npm" />
  </a>
  <a href="https://aur.archlinux.org/packages/ruffle-nightly-bin">
    <img alt="Ruffle AUR" src="https://img.shields.io/aur/version/ruffle-nightly-bin?logo=archlinux" />
  </a>
  <a href="https://discord.gg/ruffle">
    <img alt="Ruffle Discord" src="https://img.shields.io/discord/610531541889581066?label=&color=7389d8&labelColor=6a7ec2&logoColor=ffffff&logo=discord" />
  </a>
  <br />
  <strong><a href="https://ruffle.rs">website</a> | <a href="https://ruffle.rs/demo">demo</a> | <a href="https://github.com/ruffle-rs/ruffle/releases">nightly builds</a> | <a href="https://github.com/ruffle-rs/ruffle/wiki">wiki</a></strong>
</p>

# Ruffle

Ruffle is an Adobe Flash Player emulator written in the Rust programming language. Ruffle targets both the desktop and the web using WebAssembly.

## Project status

Ruffle is in the proof-of-concept stage and can currently run early Flash animations and games. Basic ActionScript 1.0/2.0 support is in place and improving; ActionScript 3.0 support is forthcoming. For more info, read the [project roadmap](https://github.com/ruffle-rs/ruffle/wiki/Roadmap).

## Using Ruffle

The easiest way to try out Ruffle is to visit the [web demo page](https://ruffle.rs/demo/), then click the "Browse..." button to load an SWF file of your choice.

[Nightly builds](https://ruffle.rs/#releases) of Ruffle are available for desktop and web platforms including the browser extension.

For more detailed instructions, see our [wiki page](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle).

## Building from source

[Follow the official guide](https://www.rust-lang.org/tools/install) to install Rust for your platform.

You must also have Java installed, and available on your PATH as `java`.

### Desktop

If you are building for a Linux platform, the following are typical dependencies:
#### Ubuntu

* libasound2-dev
* libxcb-shape0-dev
* libxcb-xfixes0-dev
* libgtk-3-dev
* libssl-dev
* libxcb-xinput-dev
* libxcb-xkb-dev
* libxcb-cursor-dev
* default-jre-headless
* cmake
* g++


Use the following command to build and run the desktop app:

`cargo run --release --package=ruffle_desktop`

To run a specific SWF file, pass the SWF path as an argument:

`cargo run --release --package=ruffle_desktop -- test.swf`

To build in debug mode, simply omit `--release` from the command.

## Homebrew

Ruffle Desktop can be built from our [Homebrew Tap](https://github.com/ruffle-rs/homebrew-ruffle/):

`brew install --HEAD ruffle-rs/ruffle/ruffle`

_Note: because it is HEAD-only, you'll need to run `brew upgrade --fetch-HEAD ruffle` each time you want to update._

### Web or Extension

Follow [the instructions in the web directory](web/README.md#building-from-source) for building
either the web or browser extension version of Ruffle.

### Scanner

If you have a collection of "real world" SWFs to test against, the scanner may be used to benchmark
ruffle's parsing capabilities. Provided with a folder and an output filename, it will attempt to read
all of the flash files and report on the success of such a task.

`cargo run --release --package=ruffle_scanner -- folder/with/swfs/ results.csv`

### Exporter

If you have a swf and would like to capture an image of it, you may use the exporter tool.
This currently requires hardware acceleration, but can be run headless (with no window).

- `cargo run --release --package=exporter -- path/to/file.swf`
- `cargo run --release --package=exporter -- path/to/file.swf path/to/screenshots --frames 5`

## Structure

- `core` contains the core emulator and common code
- `desktop` contains the desktop client (uses `wgpu-rs`)
- [`web`](web) contains the web client and browser extension (uses `wasm-bindgen`)
- `scanner` contains a utility to bulk parse swf files
- `exporter` contains a utility to generate PNG screenshots of a swf file

## Sponsors

You can support the development of Ruffle via [GitHub Sponsors](https://github.com/sponsors/ruffle-rs). Your sponsorship will help to ensure the accessibility of Flash content for the future. Thank you!

Sincere thanks to the diamond level sponsors of Ruffle:

<p align="center">
  <a href="https://www.newgrounds.com">
    <img src="https://ruffle.rs/assets/sponsors/newgrounds.png" alt="Newgrounds.com">
  </a>
  <a href="https://www.cpmstar.com">
    <img src="https://ruffle.rs/assets/sponsors/cpmstar.png" alt="CPMStar">
  </a>
  <a href="https://deepnight.net">
    <img src="https://ruffle.rs/assets/sponsors/deepnight.png" alt="Sébastien Bénard">
  </a>
  <a href="https://www.crazygames.com">
    <img src="https://ruffle.rs/assets/sponsors/crazygames.png" alt="Crazy Games">
  </a>
  <a href="https://www.coolmathgames.com">
    <img src="https://ruffle.rs/assets/sponsors/coolmathgames.png" alt="Cool Math Games">
  </a>
  <a href="https://www.nytimes.com/">
    <img src="https://ruffle.rs/assets/sponsors/nyt.png" alt="The New York Times">
  </a>
  <a href="https://www.armorgames.com/">
    <img src="https://ruffle.rs/assets/sponsors/armorgames.png" alt="Armor Games">
  </a>
  <a href="https://www.ondaeduca.com/">
    <img src="https://ruffle.rs/assets/sponsors/ondaeduca.png" alt="Onda Educa">
  </a>
  <a href="https://www.twoplayergames.org/">
    <img src="https://ruffle.rs/assets/sponsors/twoplayergames.png" alt="TwoPlayerGames.org">
  </a>
  <a href="https://www.wowgame.jp/">
    <img src="https://ruffle.rs/assets/sponsors/wowgame.png" alt="wowgame.jp">
  </a>
  <a href="http://kupogames.com/">
    <img src="https://ruffle.rs/assets/sponsors/mattroszak.png" alt="Matt Roszak">
  </a>
  <a href="https://www.dolldivine.com/">
    <img src="https://ruffle.rs/assets/sponsors/dolldivine.png" alt="Doll Divine">
  </a>
  <a href="https://movavi.com/">
    <img src="https://ruffle.rs/assets/sponsors/movavi.svg" alt="Movavi">
  </a>
  <a href="https://www.kongregate.com/">
    <img src="https://ruffle.rs/assets/sponsors/kongregate.svg" alt="Kongregate">
  </a>
</p>

## License

Ruffle is licensed under either of

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (http://opensource.org/licenses/MIT)

at your option.

Ruffle depends on third-party libraries under compatible licenses. See [LICENSE.md](LICENSE.md) for full information.

### Contribution

Ruffle welcomes contribution from everyone. See [CONTRIBUTING.md](CONTRIBUTING.md) for help getting started.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

The entire Ruffle community, including the chat room and GitHub project, is expected to abide by the [Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) that the Rust project itself follows.
