# Contributing to Ruffle

🎉 Thanks for your interest in Ruffle! Contributions of all kinds are welcome.

This document serves as a general guide for contributing to Ruffle. Follow your best judgement in following these guidelines.

## Table of Contents

 * [Getting Started](#getting-started)
 * [Ways to Contribute](#ways-to-contribute)
   * [Test your favorite Flash content](#test-your-favorite-flash-content)
   * [Improve documentation](#improve-documentation)
   * [Fix interesting issues](#fix-interesting-issues)
   * [Implement missing Flash functionality](#implement-missing-flash-functionality)
 * [Reporting Bugs](#reporting-bugs)
 * [Debugging ActionScript Content](#debugging-actionscript-content)
 * [Code Guidelines](#code-guidelines)
 * [Commit Message Guidelines](#commit-message-guidelines)
 * [Pull Requests](#pull-requests)

## Getting Started

The [Ruffle wiki](https://github.com/ruffle-rs/ruffle/wiki) is a great way to familiarize yourself with the project. It contains info on how to building Ruffle, using Ruffle, and links to helpful documentation about the Flash format.

Feel free to ask questions in our [Discord server](https://discord.gg/J8hgCQN).

## Ways to Contribute

We love new contributors! You can contribute to Ruffle in several ways:

### Test your favorite Flash content

Try out your favorite SWF content in Ruffle and see how it works! Follow the instructions on the [Using Ruffle](https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle) page to get Ruffle for your desired platform. You can run the SWF through the desktop player, the web demo, or try the extension on live websites.

If you encounter specific issues with content, please follow the guidelines on filing an issue.

### Improve documentation

Improving documentation is a great way to learn the codebase. Adding documentation to both the wiki and the code eases the learning curve for both end users and new contributors.

For documentation in the code, we follow the [rustdoc](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments) guidelines.

### Fix interesting issues

Try your hand at fixing [issues that are interesting to you](https://github.com/ruffle-rs/ruffle/issues). Follow the instructions on [building Ruffle](https://github.com/ruffle-rs/ruffle/wiki/Building-Ruffle), familiarize yourself with the [project layout](https://github.com/ruffle-rs/ruffle/wiki/Project-Layout), and use [SWF resources and decompilers](https://github.com/ruffle-rs/ruffle/wiki/Helpful-Resources) to help debug the issue.

You can also ask for mentoring on our [Discord server](https://discord.gg/J8hgCQN).

### Implement missing Flash functionality

Ruffle is a young project, and there is still much Flash functionality that is unimplemented. Check for the ["unimplemented"](https://github.com/ruffle-rs/ruffle/issues?q=is%3Aissue+is%3Aopen+label%3Aunimplemented) in issues.

## Debugging ActionScript Content

If you build Ruffle with `--features avm_debug` and enable debug logging (`RUST_LOG="warn,ruffle_core=debug,avm_trace=trace"`) then you will
activate a few built-in debugging utilities inside Ruffle, listed below.

### Warnings and Errors
All AVM errors and warnings will print their stack trace so that you can view where they are in relation to the
ActionScript inside the movie. This requires no extra configuration and will be visible by default.

### Trace statements
With `avm_trace=trace`, `trace()` statements will print to stderr.

### Step-By-Step Output
If you use the hotkey `CTRL + ALT + D` you will toggle verbose AVM debugging output on and off (default off).
You will be able to follow the flow of ActionScript inside of a SWF movie, as each action is performed.
Please note that this will likely slow down Ruffle, and it may significantly spam output. Please use sparingly.

When paired with a tool such as [JPEXS](https://github.com/jindrapetrik/jpexs-decompiler), you can compare the ActionScript
you see being executed in Ruffle with the actual ActionScript inside of the game, and attempt to find whatever problem
it is that you're looking for.

### Complete Variable Dumping
The hotkey `CTRL + ALT + V` will dump every variable inside the AVM at the moment you press it.
This can be very useful to inspect the internal state of games and see, for example, if a coordinate is NaN, your lives
are negative, or maybe an important object just didn't get initialized.


## Reporting bugs

[Issue reports and feature requests](https://github.com/ruffle-rs/ruffle/issues) are encouraged, and are a great way to measure our progress!

When filing an issue, if possible, please include:

 * A clear description of the problem
 * The platform you are testing on (web, desktop, OS)
 * A link/attachment to the SWF demonstrating the issue, if possible
 * Screenshots if the issue is a visible problem
    * Bonus points for including the correct output from the official Flash Player

These types of focused issues are helpful:

 * Tracking issues for specific Flash features (ActionScript 3.0, drawing API, etc.)
 * Bug reports for specific content that works but isn't quite right (art not looking correct, etc.)
 * Platform-specific issues
 * Enhancement requests to improve user experience

The project is still in the early stages, so many Flash features are unimplemented and not yet expected to work. Please avoid filing generic issues such as:

 * A "this SWF doesn't work at all" report (what about it doesn't work?)
 * Duplicate issues for each piece of content using an unimplemented feature
 * Asking for dates when a feature will be implemented

## Code Guidelines

Ruffle is built using the latest stable version of the Rust compiler. Nightly and unstable features should be avoided.

The Rust code in Ruffle strives to be idiomatic. The Rust compiler should emit no warnings when building the project. Additionally, all code should be formatted using [`rustfmt`](https://github.com/rust-lang/rustfmt) and linted using [`clippy`](https://github.com/rust-lang/rust-clippy). You can install these tools using `rustup`:

```sh
rustup component add rustfmt
rustup component add clippy
```

You can auto-format your changes with `rustfmt`:

```sh
cargo fmt --all
```

and you can run the clippy lints:

```sh
cargo clippy --all --tests
```

Specific warnings and clippy lints can be allowed when appropriate using attributes, such as:

```rs
#[allow(clippy::float_cmp)]
```

### Test Guidelines

Heavily algorithmic code may benefit from unit tests in Rust: create a module `mod tests` conditionally compiled with `#[cfg(test)]`, and add your tests in there.

Most tests are swf-based, with the swfs stored in `core/tests/swfs`. They are configured in `core/tests/regression_tests.rs`.

To add a test here, create a .swf file that runs `trace()` statements. You can do this by:
* creating a .fla file in a Flash authoring tool
* creating a .as file in a text editor, and compiling it using a commandline compilation tool:
    * [`mtasc`](http://web.archive.org/web/20210324063628/http://tech.motion-twin.com/mtasc.html) (ActionScript 2 only)
        * if you create a file `test.as` with a `class Test` with a `static function main` with the code you want to run, you can compile it using `mtasc -main -header 200:150:30 test.as -swf test.swf`
    * [`mxmlc`](https://helpx.adobe.com/air/kb/archived-air-sdk-version.html) (ActionScript 3 only)
        * if you create a file `test.as` with a `class Test`, you can compile it using `mxmlc Test.as`. `mxmlc` is located in the `bin` folder of the downloadable AIR SDK.
        * you may want to use docker instead -- something like `docker run -it --rm -v ${PWD}:/src jeko/airbuild mxmlc ./Test.as` works well

Run the .swf in Flash Player and create a file `output.txt` with the contents of the trace statements. Add the `output.txt`, `test.swf` and either the `test.as` or `test.fla` file to a directory under `core/tests/swfs/avm1` (or `avm2`) named after what your test tests, and add a line in `regression_tests.rs` to have Ruffle run it.

Running `cargo test [your test]` will run the .swf in Ruffle and check the `trace()` output against `output.txt`.

## Commit Message Guidelines

Here is a sample commit message:

```
web: Fix incorrect rendering of gradients (close #23)
```

 * If applicable, prefix the first line with a tag indicating the relevant area of changes:
   * `core:`
   * `desktop:`
   * `web:`
   * `avm1:`
   * `docs:`
   * `chore:`
   * `tests:`
 * Capitalize the first letter following the tag
 * Limit line length to 72 characters
 * Use the present tense and imperative mood ("fix", not "fixed" nor "fixes")
 * Reference any PRs or issues in the first line
 * Use keywords to close/address issues when applicable ("close #23")
 * Write more detailed info on following lines when applicable

## Pull Requests

Pull requests are the primary way to contribute code to Ruffle. Pull requests should be made against the latest `master` branch. Your pull request should not contain merges; you should always rebase when bringing the latest changes into your branch from the `master` branch. If there are merge conflicts, or if your commit history is messy, please rebase onto the latest master. [`git rebase -i`](https://thoughtbot.com/blog/git-interactive-rebase-squash-amend-rewriting-history#interactive-rebase) is a great way to clean up your pull request.

When you make a pull request, our [CI](https://circleci.com/gh/ruffle-rs/ruffle) will build your changes and run them through all tests and style checks. All of these tests should pass before your pull request can be accepted.

One of [our regular contributors](https://github.com/orgs/ruffle-rs/people) will review your changes and try their best to helpfully suggest any changes. If all goes well, your PR should be merged without much delay. We use both standard merge commits and fast-forward merges depending on the size of the changes. Thanks for your contribution!
