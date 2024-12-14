<p align="center">
  <a href="https://ruffle.rs"><img alt="Ruffle" src="https://ruffle.rs/logo.svg" /></a>
</p>
<p align="center">
  <a href="https://github.com/ruffle-rs/ruffle/actions"><img alt="Rust Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_rust.yml?label=Rust%20Build&logo=github&branch=master" /></a>
  <a href="https://github.com/ruffle-rs/ruffle/actions/workflows/test_web.yml"><img alt="Web Build Status" src="https://img.shields.io/github/actions/workflow/status/ruffle-rs/ruffle/test_web.yml?label=Web%20Build&logo=github&branch=master" /></a>
  <a href="https://flathub.org/apps/rs.ruffle.Ruffle"><img alt="Ruffle Flathub" src="https://img.shields.io/flathub/v/rs.ruffle.Ruffle?color=007acc&logo=flathub" /></a>
  <a href="https://www.npmjs.com/package/@ruffle-rs/ruffle"><img alt="Ruffle npm" src="https://img.shields.io/npm/v/@ruffle-rs/ruffle?color=007acc&logo=npm" /></a>
  <a href="https://aur.archlinux.org/packages/ruffle-nightly-bin"><img alt="Ruffle AUR" src="https://img.shields.io/aur/version/ruffle-nightly-bin?logo=archlinux" /></a>
  <a href="https://discord.gg/ruffle"><img alt="Ruffle Discord" src="https://img.shields.io/discord/610531541889581066?label=&color=7389d8&labelColor=6a7ec2&logoColor=ffffff&logo=discord" /></a>
  <a href="https://crowdin.com/project/ruffle"><img alt="Ruffle translations" src="https://badges.crowdin.net/ruffle/localized.svg" /></a>
  <br />
  <strong><a href="https://ruffle.rs">website</a> | <a href="https://ruffle.rs/demo">demo</a> | <a href="https://github.com/ruffle-rs/ruffle/releases">nightly builds</a> | <a href="https://github.com/ruffle-rs/ruffle/wiki">wiki</a></strong>
</p>

# Ruffle

Ruffle is an Adobe Flash Player emulator written in the Rust programming language. Ruffle targets both the desktop and the web using WebAssembly.

## Table of Contents
* [Project status](#project-status)
* [Using Ruffle](#using-ruffle)
* [Building from source](#building-from-source)
  * [Prerequisites](#prerequisites)
  * [Linux prerequisites](#linux-prerequisites)
  * [Desktop](#desktop)
    * [Build](#build)
    * [macOS](#macos)
  * [Web or Extension](#web-or-extension)
  * [Android](#android)
  * [Scanner](#scanner)
  * [Exporter](#exporter)
* [Structure](#structure)
* [Sponsors](#sponsors)
* [License](#license)
* [Contributing](#contributing)


## Project status

Ruffle supports ActionScript 1, 2 and 3 pretty well, but it's still not finished by any means. Please report any issues in the [Issue Tracker](https://github.com/ruffle-rs/ruffle/issues).

## Using Ruffle

The easiest way to try out Ruffle is to visit the [web demo page](https://ruffle.rs/demo/), then click the "Select File" button to load a SWF file of your choice.

[Nightly builds](https://ruffle.rs/downloads#nightly-releases) of Ruffle are available for desktop and web platforms.

For more detailed instructions, see our [wiki page](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle).

## Building from source

### Prerequisites

* Latest stable channel of [Rust](https://www.rust-lang.org/tools/install)
* Java, available on your PATH as `java` (required for building the library containing the builtin Flash classes for ActionScript 3)

### Linux prerequisites

The following are typical dependencies for Linux:

* libasound2-dev
* libxcb-shape0-dev
* libxcb-xfixes0-dev
* libgtk-3-dev
* libudev-dev
* libxcb-xinput-dev
* libxcb-xkb-dev
* libxcb-cursor-dev
* default-jre-headless
* cmake
* g++

### Desktop

#### Build

Use the following command to build and run the desktop app:

`cargo run --release --package=ruffle_desktop`

To run a specific SWF file, pass the SWF path as an argument:

`cargo run --release --package=ruffle_desktop -- test.swf`

To build in debug mode, simply omit `--release` from the command.

#### macOS

Ruffle desktop can be built from our [Homebrew Tap](https://github.com/ruffle-rs/homebrew-ruffle/):

`brew install --HEAD ruffle-rs/ruffle/ruffle`

_Note: because it is HEAD-only, you'll need to run `brew upgrade --fetch-HEAD ruffle` each time you want to update._

### Web or Extension

Follow [the instructions in the web directory](web/README.md#building-from-source) for building
either the web or browser extension version of Ruffle.

This project is tested with BrowserStack.

### Android

Follow the [instructions](https://github.com/ruffle-rs/ruffle-android/blob/main/CONTRIBUTING.md#building-from-source) in the `ruffle-android` project for building the Android application of Ruffle.

### Scanner

If you have a collection of "real world" SWFs to test against, the scanner may be used to benchmark
ruffle's parsing capabilities. Provided with a folder and an output filename, it will attempt to read
all of the Flash files and report on the success of such a task.

`cargo run --release --package=ruffle_scanner -- scan folder/with/swfs/ results.csv`

### Exporter

If you have a SWF file and would like to capture an image of it, you may use the exporter tool.
This currently requires hardware acceleration, but can be run headless (with no window).

- `cargo run --release --package=exporter -- path/to/file.swf`
- `cargo run --release --package=exporter -- path/to/file.swf path/to/screenshots --frames 5`

## Structure

- `core` - core emulator and common code
- `swf` - SWF and ActionScript parser
- `desktop` - desktop client (uses `wgpu-rs`)
- `web` - web client and browser extension (uses `wasm-bindgen`)
- `render` - various rendering backends for both desktop and web
- `video` - video decoding backends
- `flv` - Flash Video decoder
- `wstr` - a Flash-compatible implementation of strings
- `scanner` - a utility to bulk parse SWF files
- `exporter` - a utility to generate PNG screenshots of a SWF file

## Sponsors

You can support the development of Ruffle via [GitHub Sponsors](https://github.com/sponsors/ruffle-rs). Your sponsorship will help to ensure the accessibility of Flash content for the future. Thank you!

Sincere thanks to the diamond level sponsors of Ruffle:

<p align="center">
  <a href="https://www.newgrounds.com"><img src="https://ruffle.rs/sponsors/newgrounds.png" alt="Newgrounds.com"></a>
  <a href="https://www.cpmstar.com"><img src="https://ruffle.rs/sponsors/cpmstar.png" alt="CPMStar"></a>
  <a href="https://deepnight.net"><img src="https://ruffle.rs/sponsors/deepnight.png" alt="Sébastien Bénard"></a>
  <a href="https://www.crazygames.com"><img src="https://ruffle.rs/sponsors/crazygames.png" alt="Crazy Games"></a>
  <a href="https://www.coolmathgames.com"><img src="https://ruffle.rs/sponsors/coolmathgames.png" alt="Cool Math Games"></a>
  <a href="https://www.nytimes.com/"><img src="https://ruffle.rs/sponsors/nyt.png" alt="The New York Times"></a>
  <a href="https://www.armorgames.com/"><img src="https://ruffle.rs/sponsors/armorgames.png" alt="Armor Games"></a>
  <a href="https://www.ondaeduca.com/"><img src="https://ruffle.rs/sponsors/ondaeduca.png" alt="Onda Educa"></a>
  <a href="https://www.twoplayergames.org/"><img src="https://ruffle.rs/sponsors/twoplayergames.png" alt="TwoPlayerGames.org"></a>
  <a href="https://www.wowgame.jp/"><img src="https://ruffle.rs/sponsors/wowgame.png" alt="wowgame.jp"></a>
  <a href="http://kupogames.com/"><img src="https://ruffle.rs/sponsors/mattroszak.png" alt="Matt Roszak"></a>
  <a href="https://www.dolldivine.com/"><img src="https://ruffle.rs/sponsors/dolldivine.png" alt="Doll Divine"></a>
  <a href="https://movavi.com/"><img src="https://ruffle.rs/sponsors/movavi.svg" alt="Movavi"></a>
  <a href="https://www.kongregate.com/"><img src="https://ruffle.rs/sponsors/kongregate.svg" alt="Kongregate"></a>
  <a href="https://www.bubbleshooter.net/"><img src="https://ruffle.rs/sponsors/bubble-shooter.png" alt="Bubble Shooter"></a>
  <a href="https://www.neopets.com/"><img src="https://ruffle.rs/sponsors/neopets.png" alt="Neopets"></a>
</p>

## License

Ruffle is licensed under either of

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (http://opensource.org/licenses/MIT)

at your option.

Ruffle depends on third-party libraries under compatible licenses. See [LICENSE.md](LICENSE.md) for full information.

### Contributing

Ruffle welcomes contribution from everyone. See [CONTRIBUTING.md](CONTRIBUTING.md) for help getting started.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

The entire Ruffle community, including the chat room and GitHub project, is expected to abide by the [Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) that the Rust project itself follows.
