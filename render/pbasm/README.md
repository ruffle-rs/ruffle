# pbasm: The Last PixelBender (Assembler)

`pbasm` is a PixelBender assembler with a custom syntax.

## Usage

In order to assemble a `.pbasm` file, run:

```bash
pbasm kernel.pbasm -o kernel.pbj
```

In order to disassemble a `.pbj` file, run:

```bash
pbasm -d kernel.pbj -o kernel.pbasm
```

## Syntax

See [integration tests](./integration_tests/) for syntax examples.

## Notes

`pbasm` aims mainly to be useful for Ruffle developers, which means that it
should be able to produce malformed shaders too, so that we can make sure they
are handled properly.
