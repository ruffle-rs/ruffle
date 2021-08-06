# ruffle-web

![Test Web](https://github.com/ruffle-rs/ruffle/workflows/Test%20Web/badge.svg)

ruffle-web is a Wasm version of Ruffle, intended for use by either
using the `ruffle-selfhosted` or `ruffle-extension` NPM packages.

This project is split into two parts: The actual Flash player written in Rust,
and a javascript interface to it. Most of the time, you will be building the
actual rust part through the npm build scripts.

## Using ruffle-web

Please refer to our wiki for instructions on how to use Ruffle either
[on your own website](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#web),
or as a [browser extension](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#browser-extension).

## How it works

We compile Ruffle down to a Wasm ([WebAssembly](https://webassembly.org/)) binary, which will be loaded
into web pages either deliberately (installing the selfhosted package onto the website), or injected
by users as a browser extension.

By default we will detect and replace any embedded Flash content on websites with the Ruffle player
(we call this "polyfilling"), but this can be configured by the website. This means that Ruffle is an
"out of the box" solution for getting Flash to work again; include Ruffle and it should just work™.

For rendering the content, we prefer to use [WebGL](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API).
WebGL is very accurate, hardware-accelerated and very fast, but is not universally supported.
Additionally, many privacy related browsers or extensions will disable WebGL by default.
For this reason, we include a fallback using [the canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API).

## Building from source

### Requirements

Before you are able to build this project, you must first install a few dependencies:

#### Rust

Follow the instructions [to install rust](https://www.rust-lang.org/tools/install) on your machine.

We do not have a Minimum Supported Rust Version policy. If it fails to build, it's likely you may need
to update to the latest stable version of rust. You may run `rustup update` to do this (if you installed
rust using the above instructions).

For the compiler to be able to output WebAssembly, an additional target has to be added to it: `rustup target add wasm32-unknown-unknown`

#### Node.js

Follow the instructions [to install node.js](https://nodejs.org/en/) on your machine.

We recommend using the currently active LTS 14, but we do also run tests with maintenance LTS 12.

#### wasm-bindgen

<!-- Be sure to also update the wasm-bindgen-cli version in .github/workflows/*.yaml and web/Cargo.toml -->
This can be installed with `cargo install wasm-bindgen-cli --version 0.2.75`. Be sure to install this specific version of `wasm-bindgen-cli` to match the version used by Ruffle.

#### Binaryen

This is optional, used to further optimize the built WebAssembly module.
Some ways to install Binaryen:

-   download one of the [prebuilt releases](https://github.com/WebAssembly/binaryen/releases/)
-   using your Linux distribution's package manager (`sudo apt install binaryen`, `sudo dnf install binaryen`)
-   from [Homebrew](https://formulae.brew.sh/formula/binaryen)
-   from [Anaconda](https://anaconda.org/conda-forge/binaryen)
-   [compile it yourself](https://github.com/WebAssembly/binaryen#building)

Just make sure the `wasm-opt` program is in `$PATH`, and that it works.

### Building

In this project, you may run the following commands to build all packages:

-   `npm run bootstrap`
    -   This will install every dependency for every package.
    -   Run this every time you pull in new changes, otherwise you may be missing a package and the build will fail.
-   `npm run build`
    -   This will build the wasm binary and every node package (notably selfhosted and extension).
    -   Output will be available in the `dist/` of each package (for example, `./packages/selfhosted/dist`),
        save for the extension which is directory `build/`.
    -   You may also use `npm run build:debug` to disable Webpack optimizations and activate the (extremely verbose) ActionScript debugging output.

From here, you may follow the instructions to [use Ruffle on your website](packages/selfhosted/README.md),
or run a demo locally with `npm run demo`.

### Testing

To run all of the tests in this project, we currently require that you have [Chrome installed to its default location](https://www.google.com/chrome/).

First, ensure you've build every package (see above), and then run `npm run test` to run the full suite of tests.

## Structure

-   This directory is a cargo crate which is the actual Flash player, and also a root node package.
-   [packages/core](packages/core) is a node package which contains the core ruffle web API & wasm bindings.
-   [packages/selfhosted](packages/selfhosted) is a node package intended for consumption by websites to include Ruffle on their site.
-   [packages/extension](packages/extension) is a node package that turns Ruffle into a browser extension.
-   [packages/demo](packages/demo) is an example node package of how to use self-hosted ruffle on your site, and testing it locally.

## Contributing

Please follow the [general contribution guidelines for Ruffle](../CONTRIBUTING.md).

In addition to those, we ask that you ensure that you pass all tests with `npm run test`, and check the automatic code
linting & styler by running `npm run format` before you commit.

Where possible, please add tests to all new functionality or bug fixes that you contribute.

Thank you!
