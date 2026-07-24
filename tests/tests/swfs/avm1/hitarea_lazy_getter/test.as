// Tests that hitArea resolves through a full, lazy property get at pick
// time: the getter lives on MovieClip.prototype (so lookup must walk the
// prototype chain), the setter consumes every assignment without storing,
// and picking must follow the getter's CURRENT return value. A write-time
// snapshot implementation (or one reading only own properties) fails both
// halves: nothing was ever stored, and the hit area switches from clipA to
// clipB mid-run. The getter is deliberately trace-quiet because Flash's
// poll cadence is nondeterministic; evidence comes from rollovers only.
//
// Mouse path (stage coordinates; every move is followed by one frame of
// Wait): start (0,0); (250,100) over clipB while the getter still
// returns clipA -> silent; (450,100) over btn's own (replaced) shape ->
// silent; (450,300) over the consumed decoy -> silent; (100,100) over
// clipA -> rollover A + switch; out (0,0); (100,100) again, now that
// the getter returns clipB -> silent; out (0,0); (250,100) over clipB
// -> rollover B.
//
// Separate from hitarea_sweep because the prototype-level setter here
// would consume every plain hitArea assignment in a shared SWF.
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

box("btn", 1, 0xFF0000, 400, 50, 100, 100);    // red: btn's own (replaced) shape
box("clipA", 2, 0x0000FF, 50, 50, 100, 100);   // blue: getter's phase-1 target
box("clipB", 3, 0x00FF00, 200, 50, 100, 100);  // green: getter's phase-2 target
box("decoy", 4, 0xFF8800, 400, 250, 100, 100); // orange: assigned but consumed

var phase = 1;
MovieClip.prototype.addProperty("hitArea",
    function() { return phase == 1 ? _root.clipA : _root.clipB; },
    function(v) { trace("setter consumed " + v); });

btn.hitArea = decoy;
trace("readback: " + btn.hitArea);

btn.onRollOver = function() {
    var x = _root._xmouse;
    var y = _root._ymouse;
    var zone = y > 200 ? "decoy" : (x < 175 ? "A" : (x < 350 ? "B" : "own"));
    trace("rollover " + zone);
    if (zone == "A" && phase == 1) {
        phase = 2;
        trace("switch");
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
