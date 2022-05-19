# Ruffle AVM2 globals

This directory contains implementations of Flash global
definitions (e.g. `Object`, `flash.geom.Point`, `flash.utils.getDefinitionByName`)

Currently, globals are implemented in one of two ways:
1) As pure-Rust code in files like `system.rs`. These are normal Rust
   modules, and are used from `core/src/avm2/global.rs`
2) As ActionScript class definitions like `flash/geom/Point.as`.
   These files are automatically compiled into a `playerglobal.swf`
   file at build time, which is included into the final Ruffle binary
   and loaded during player initialization.

In many cases, defining a class in ActionScript results in
code that's much simpler and more readable than if were
defined in Rust.

Flash's `playerglobal.swc` (specifically, its `library.swf`)
cannot be used as a drop-in replacement for our `playerglobal.swf`.
In addition to potential copyright issues around redistributing Flash's `playerglobal.swc`,
many of its classes rely on specific 'native' methods being provided
by the Flash VM, which Ruffle does not implement.

## Compiling

Java must be installed for the build process to complete.

ActionScript classes are processed by the `core/build\_playerglobal`

The tool first uses 'asc.jar'
from the Flex SDK to compile these ActionScript files into
ABC (bytecode). This tool is entirely self-contained, so we can
include in our repository without requiring the entire Flex SDK
to be installed.

The produced ABC files are then combined into the final
`playerglobal.swf` (which is written out into the build directory,
and not checked in to Git).

The `core/build\_playerglobal` tool is automatically run by `core`'s build script
when any of our ActionScript classes are changed.

NOTE - when adding a *new* .as file, you must manually trigger
the build script to be re-run by running 'touch core/build.rs'
(or otherwise updating the modification time of 'core/build.rs').
This only needs to be done once - afterwards, Cargo will track
changes to your newly added '.as' file, and re-run the build
script as needed.

## Limitations

* Only pure ActionScript classes are currently supported. Classes with
'native' methods are not yet supported.
* 'Special' classes which are loaded early during Ruffle initialization
(e.g. 'Object', 'Function', 'Class') cannot currently
be implemented in 'playerglobal', since Ruffle initializes them in a special
way. However, virtually all classes in the 'flash' package are initialized
in a 'normal' way, and are eligible for implementation in 'playerglobal'
