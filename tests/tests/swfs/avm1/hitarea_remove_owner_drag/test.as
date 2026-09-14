// Tests _droptarget across a mid-pick self-removal: btnRm's own hitArea
// getter, once armed by a click, removes btnRm during the drop-target pick
// in which the drag enters the left zone. Flash Player still reports the
// just-removed clip on that pick (dt: /btnRm) — removal only takes effect
// on subsequent picks, which land on the underlying btnZ (dt: /btnZ).
// A post-drag hover confirms events go to btnZ afterwards (rollover Z).
//
// Mouse path (stage coordinates; every move is followed by one frame of
// Wait): start (450,300); hover btnRm (100,100) -> rollover Rm; out
// (450,300); over drag (275,200); button down -> armed + drag start;
// (220,160); (100,100) -> removing owner, dt: /btnRm, then dt: /btnZ;
// (130,120); button up -> drag end; (450,300); (100,100) -> rollover Z;
// (450,300).
//
// Separate from hitarea_sweep for the same precaution as remove_sibling:
// Flash Player froze after a mid-pick sibling removal in earlier
// captures, and stacking removal cases in one capture would compound
// that risk.
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

box("btnZ", 1, 0x00FF00, 50, 50, 100, 100);  // green, UNDER btnRm: live target
box("btnRm", 2, 0x0000FF, 50, 50, 100, 100); // blue, on top: removes itself
box("drag", 3, 0xFFFF00, 260, 185, 30, 30);  // yellow, center: draggable

var armed = false;
var removed_done = false;

btnRm.addProperty("hitArea", function() {
    if (armed && !removed_done && _root._xmouse < 175) {
        removed_done = true;
        trace("removing owner");
        _root.btnRm.removeMovieClip();
    }
    return undefined;
}, null);

btnRm.onRollOver = function() {
    trace("rollover Rm");
};
btnZ.onRollOver = function() {
    trace("rollover Z");
};

onMouseDown = function() {
    if (!armed) {
        armed = true;
        trace("armed");
    }
};

drag.onPress = function() {
    trace("drag start");
    this.startDrag(false);
};
drag.onRelease = function() {
    this.stopDrag();
    trace("drag end");
};
drag.onReleaseOutside = drag.onRelease;

var last = "(init)";
trace("ready");

var frames = 0;
onEnterFrame = function() {
    frames++;
    var dt = drag._droptarget;
    if (dt != last) {
        last = dt;
        trace("dt: " + (dt == "" ? "(none)" : dt));
    }
    if (frames >= 180) {
        fscommand("quit");
    }
};
