// Tests that no rollover is delivered for a clip removed mid-pick by a
// sibling's getter: btnX (higher depth, visited first) has a hitArea
// getter that, once armed by a click, removes btnY during the pick in
// which the mouse enters btnY's zone. The removal is position-gated so
// it can only fire during that pick. Expected: btnY's first visit fires
// a rollover; after arming, the visit that removes it delivers none.
//
// Mouse path (stage coordinates; every event is followed by one frame
// of Wait): start (0,0); hover btnY (100,100) -> rollover Y; out (0,0);
// click at (0,0) -> armed; (100,100) again -> btnX's getter removes
// btnY during this pick and no rollover is delivered.
//
// Separate from hitarea_sweep as a precaution: Flash Player froze after
// a mid-pick sibling removal in earlier captures, and a second removal
// case in the sweep's capture would compound that risk.
stop();

function box(name, depth, color, x, y, w, h) {
    var mc = _root.createEmptyMovieClip(name, depth);
    mc.beginFill(color);
    mc.moveTo(x, y);
    mc.lineTo(x + w, y);
    mc.lineTo(x + w, y + h);
    mc.lineTo(x, y + h);
    mc.endFill();
    return mc;
}

box("btnY", 1, 0x0000FF, 50, 50, 100, 100);  // blue, left, lower depth: victim
box("btnX", 2, 0xFF0000, 400, 50, 100, 100); // red, right, higher depth: owner

var armed = false;
var removed_done = false;

btnX.addProperty("hitArea", function() {
    if (armed && !removed_done && _root._xmouse < 175) {
        removed_done = true;
        trace("removing sibling");
        _root.btnY.removeMovieClip();
    }
    return undefined;
}, null);
btnX.onRollOver = function() {};

btnY.onRollOver = function() {
    trace("rollover Y");
};

onMouseDown = function() {
    if (!armed) {
        armed = true;
        trace("armed");
    }
};

trace("ready");

var frames = 0;
onEnterFrame = function() {
    frames++;
    if (frames >= 239) {
        fscommand("quit");
    }
};
