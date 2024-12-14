# AVM2 globals

This directory contains implementations of global definitions
(e.g. `Object`, `flash.geom.Point`, `flash.utils.getDefinitionByName`).

WARNING: Do *not* implement classes by copying their (decompiled) ActionScript
from the Adobe Flash `playerglobal.swf`. This would violate copyright by making
Ruffle re-distribute Adobe's code (and will not even work in general, since
Adobe's `playerglobal.swf` uses native methods that Ruffle doesn't implement).

Currently, globals are implemented in one of two ways:
1) As pure-Rust code in files like `system.rs`. These are normal Rust
   modules, and are used from `core/src/avm2/global.rs`.
2) As ActionScript code like `flash/geom/Point.as`.
   The files included from `globals.as` are compiled into a `playerglobal.swf`
   file at build time, which is included in the final Ruffle binary
   and loaded during player initialization.

ActionScript files included from `stubs.as` can be referenced from other `.as` files,
but they will not be included in the final `playerglobal.swf`. This is useful when you need to write
an `.as` file that references a class defined in Rust - you can create
a stub class without needing to port the entire pre-existing class
to ActionScript.

In many cases, defining a class in ActionScript results in
code that's much simpler and more readable than if were
defined in Rust.

Flash's `playerglobal.swc` (specifically, its `library.swf`)
cannot be used as a drop-in replacement for our `playerglobal.swf`.
In addition to potential copyright issues around redistributing Flash's `playerglobal.swc`,
many of its classes rely on specific 'native' methods being provided
by the Flash VM, which Ruffle does not implement.

## Calling AS3 methods

Under some circumstances, it may be necessarily to call a method with an explicitly-qualified AS3 namespace:

```
var xml = <accessor />;
xml.AS3::appendChild(elem);
```

In order for this to generate efficient bytecode, you must have `namespace AS3 = "http://adobe.com/AS3/2006/builtin";` inside
your package declaration (see `GroupElement.as` for an example). If you forget to do this, you'll get an error at compile-time:

Found getlex of "AS3" in method body. Make sure you have `namespace AS3 = "http://adobe.com/AS3/2006/builtin";` in your `package` block

## Native methods

We support defining native methods (instance methods, class methods, and freestanding functions)
in ActionScript classes in playerglobal. During the build process, we automatically
generate a reference to a Rust function at the corresponding path in Ruffle.

For example, the native method function `flash.system.Security.allowDomain`
expects a Rust function to be defined at `crate::avm2::globals::flash::system::security::allow_domain`.

This function is cast to a `NativeMethodImpl` function pointer, exactly like
functions defined on a pure-Rust class definition.

If you're unsure of the path to use, just build Ruffle after marking the
`ActionScript` method as `native` - the compiler will produce an error
explaining where the Rust function needs to be defined.

The ActionScript method and the Rust function are automatically linked
together, and the Rust function will be invoked when the corresponding
function is called from ActionScript.

## Custom instance allocator

You can use a custom instance allocator method by applying the metadata
`[Ruffle(InstanceAllocator)]`
to your class definition. A reference to a function named `<classname>_allocator`
will be generated - this should be an `AllocatorFn`, just like when defining
a class in Rust. This allocator will automatically be registered when the corresponding
class is loaded.

See `flash/events/Event.as` for an example

## API Versioning

Ruffle supports Flash's API versioning, which hides newer playerglobal definitions
(including methods/properties) from SWFs compiled with older API versions.
For example, see `Event.WORKER_STATE`

To add versioning to an API:

1. Determine the first version where it was added. This can be seen in the Flash Documentation (e.g. "Runtime Versions: Flash Player 11.4, AIR 3.4")
2. Convert the Flash Player version to an SWF version number using [this chart](https://github.com/ruffle-rs/ruffle/wiki/SWF-version-chart)
2. Determine the corresponding asc.jar version code for the SWF version. This can be found in avmplus in https://github.com/adobe/avmplus/blob/master/core/api-versions.as
3. Add an `[API("VersionCode")]` metadata to the definition. In the `Event.WORKER_STATE` example,
   this looks like:

   ```actionscript
   [API("682")]
   public static const WORKER_STATE:String = "workerState";
   ```

   WORKER_STATE was added in Flash Player 11.4, which corresponds to SWF version 17. Looking at the avmplus file, this corresponds
   to a version code of "682".

## Compiling

Java must be installed for the build process to complete.

ActionScript classes are processed by the `core/build_playerglobal`

The tool first uses 'asc.jar'
from the Flex SDK to compile these ActionScript files into
ABC (bytecode). This tool is entirely self-contained, so we can
include in our repository without requiring the entire Flex SDK
to be installed.

The produced ABC files are then combined into the final
`playerglobal.swf` (which is written out into the build directory,
and not checked in to Git).

The `core/build_playerglobal` tool is automatically run by `core`'s build script
when any of our ActionScript classes are changed.

## Limitations

* 'Special' classes which are loaded early during player initialization
(e.g. `Object`, `Function`, `Class`) cannot currently
be implemented in `playerglobal`, since they are initialized in a special
way. However, virtually all classes in the `flash` package are initialized
in a 'normal' way, and are eligible for implementation in `playerglobal`.
