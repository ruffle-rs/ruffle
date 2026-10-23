# swfasm

A textual representation of SWF files, with a compiler that turns it into SWF
files.

```sh
swfasm test.swfasm         # writes test.swf
swfasm test.swfasm -o out.swf
```

## Format

Sources use the `.swfasm` extension and are written in
[RON](https://github.com/ron-rs/ron), with the `implicit_some` extension enabled
(optional values don't need `Some(...)`).
See [`src/source.rs`](src/source.rs) for everything that is supported.

### Tags

Tags are named after SWF tags, see `Tag` in [`src/source.rs`](src/source.rs)
for the ones that are supported. In addition, there are special tags that give
control over the raw encoding:

* `Raw(code: 26, data: "01 02 ff")` emits a tag with an arbitrary code and a
  body given as [binary data](#binary-data). Set `force_long: true` to use the
  long (6 byte) header form even for a short body.

### AVM1 actions

`DoAction(Bytes("96 02 00 05 01 26"))` takes AVM1 bytecode as
[binary data](#binary-data).

### Binary data

Raw binary data is written as a hex string with optional whitespace
(`"01 02 ff"`), or a list of hex strings, which are concatenated. The list form
allows commenting individual parts:

```ron
Raw(code: 1, data: [
    "0a 00", // frame count
    "ff ff", // flags
]),
```

## TODO

* Symbolic character IDs (e.g. `@name`) instead of numbers.
* Remaining tags (a lot of them).
* Loading binary data from files (e.g. images and sounds), with paths relative
  to the `.swfasm` file.
* Compiling ActionScript into tags: Rascal for AVM1 (`DoAction`,
  `DoInitAction`), asc for AVM2 (`DoAbc`).
* Compiling `.swfasm` sources in the test framework (`test.toml`).
