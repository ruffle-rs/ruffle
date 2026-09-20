// This file is executed in the global scope, after native globals are initialized
// but before any user content.
// You should not reference any display objects here; this is not going to be loaded as a "real" movie.

trace = ASnative(100, 4);

#include "flash/geom/Point.as"

// The `flash` package is only visible to SWF 8 and later.
ASSetPropFlags(_global, "flash", 4096);

// The variable `o` is being used to make referring to symbols more concise.
// However, in Flash it's not being deleted, but instead set to `null`,
// which means that in every SWF, the variable `o` is `null` and not `undefined`.
var o = null;
