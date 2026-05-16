// This is compiled as part of the build step for ruffle_core via `build_playerglobal`.
// The globals are loaded after the "built in" globals (read: things that haven't been moved here yet, and `ASnative`)
// but before any user content.
// You should not reference any display objects here; this is not going to be loaded as a "real" movie.

// For convenience, the use of `#include "foo.as"` is supported.
