# ![Ruffle](https://ruffle.rs/assets/logo.png)

Ruffle is an Adobe Flash Player emulator written in Rust.

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
