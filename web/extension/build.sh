#!/bin/bash

#Really hacky way to build everything until I figure out how to get a better/
#dependency-respecting build system going

rm -r dist
wasm-pack build --target=no-modules --out-dir=extension/dist ..
npx webpack