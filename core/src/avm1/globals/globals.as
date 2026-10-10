// This file is executed in the global scope, after native globals are initialized
// but before any user content.
// You should not reference any display objects here; this is not going to be loaded as a "real" movie.

trace = ASnative(100, 4);

#include "AsBroadcaster.as"

AsBroadcaster.initialize(Selection);
AsBroadcaster.initialize(Mouse);
AsBroadcaster.initialize(Key);
AsBroadcaster.initialize(TextField.prototype);
AsBroadcaster.initialize(Stage);
AsBroadcaster.initialize(System.IME);
AsBroadcaster.initialize(MovieClipLoader.prototype);
AsBroadcaster.initialize(flash.net.FileReference.prototype);
AsBroadcaster.initialize(flash.net.FileReferenceList.prototype);

#include "ContextMenuItem.as"
#include "ContextMenu.as"
#include "Error.as"

#include "flash/geom/Rectangle.as"
#include "flash/geom/Point.as"
#include "flash/geom/Matrix.as"

// The `flash` package is only visible to SWF 8 and later.
ASSetPropFlags(_global, "flash", 4096);

// The variable `o` is being used to make referring to symbols more concise.
// However, in Flash it's not being deleted, but instead set to `null`,
// which means that in every SWF, the variable `o` is `null` and not `undefined`.
var o = null;
