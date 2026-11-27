// Tests listeners and Color objects that were given a clip whose clip is then
// removed, or renamed, and recreated at the same path. Each case traces which
// clip gets the broadcast or the color.

function square(name, depth) {
    var mc = _root.createEmptyMovieClip(name, depth);
    mc.beginFill(0xFF0000);
    mc.moveTo(0, 0);
    mc.lineTo(10, 0);
    mc.lineTo(10, 10);
    mc.lineTo(0, 10);
    mc.endFill();

    return mc;
}

// A listener clip that is removed and then recreated at its path.
var broadcaster = {};
AsBroadcaster.initialize(broadcaster);

var listener = square("listener", 1);
listener.onFoo = function() {
    trace("onFoo: first clip, this = " + this);
};
broadcaster.addListener(listener);
broadcaster.broadcastMessage("onFoo");

listener.removeMovieClip();
trace("after removing: broadcastMessage returned " + broadcaster.broadcastMessage("onFoo"));

var recreated = square("listener", 1);
recreated.onFoo = function() {
    trace("onFoo: recreated clip, this = " + this);
};
trace("after recreating: broadcastMessage returned " + broadcaster.broadcastMessage("onFoo"));
trace("removeListener(recreated): " + broadcaster.removeListener(recreated) + ", length " + broadcaster._listeners.length);

// A listener clip that is renamed, with a new clip at its old path.
var renameBroadcaster = {};
AsBroadcaster.initialize(renameBroadcaster);

var renamed = square("renamed", 2);
renamed.onFoo = function() {
    trace("onFoo: renamed clip, this = " + this);
};
renameBroadcaster.addListener(renamed);
renamed._name = "renamedMoved";

var atOldName = square("renamed", 3);
atOldName.onFoo = function() {
    trace("onFoo: clip at the old name, this = " + this);
};
renameBroadcaster.broadcastMessage("onFoo");

// A Color object whose clip is removed and then recreated at its path.
var colored = square("colored", 4);
var color = new Color(colored);
colored.removeMovieClip();

var recreatedColored = square("colored", 4);
color.setRGB(0x00FF00);
var recreatedColor = new Color(recreatedColored);
trace("color of the recreated clip: " + recreatedColor.getRGB().toString(16));

trace("Done");
