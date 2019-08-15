<p align="center">
 <a href="https://ruffle.rs"><img src="https://ruffle.rs/assets/logo.png" alt="Ruffle"></a>
</p>
<p align="center">
 <a href="https://travis-ci.org/ruffle-rs/ruffle">
  <img src="https://img.shields.io/circleci/build/github/ruffle-rs/ruffle" alt="Travis Build Status">
 </a>
  <a href="https://discord.gg/J8hgCQN">
      <img src="https://img.shields.io/discord/610531541889581066" alt="Ruffle Discord">
  </a>
  <br>
  <strong><a href="https://ruffle.rs">website</a> | <a href="https://ruffle.rs/demo?file=heroes_of_cybertron.swf">demo</a> </strong>
</p>

# Ruffle

Ruffle is an Adobe Flash Player emulator written in the Rust programming language. Ruffle targets both the desktop and the web using WebAssembly.

## Project status

Ruffle is in the proof-of-concept stage, and can currently run early Flash animations. ActionScript support is still forthcoming; for more info, read the [project roadmap](https://github.com/ruffle-rs/ruffle/wiki/Roadmap).

## Building
 
[Follow the official guide](https://www.rust-lang.org/tools/install) to install Rust for your platform.

### Desktop

* `cargo run --package=ruffle_desktop -- test.swf`

### Web

* Install [Node.js](https://nodejs.org/en/)
* Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

#### Running the web demo

* `cd web/demo`
* `npm install`
* `npm run serve`
* Load indicated page in browser (usually http://localhost:8080)

#### Building the NPM package

* `cd web`
* `wasm-pack build`

## Structure

- `core` contains the core emulator and common code
- `desktop` contains the desktop client (uses `glium`)
- `web` contains the web client (uses `wasm-bindgen`)

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
