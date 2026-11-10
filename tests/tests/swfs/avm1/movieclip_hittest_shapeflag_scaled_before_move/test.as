// Like the obstacles of Missile Game 3D: every frame, a shape is scaled, hit tested
// with hitTest(x, y, true), and only then moved. Flash tests the shape as it was last
// drawn, so the point (the "missile" at the stage centre) stays in the gap it was
// steered into. Testing the new scale at the old position would hit the solid middle.

// A 200x200 square centred on its origin, with a gap from (40,40) to (100,100).
var ob = this.createEmptyMovieClip("ob", 1);
ob.beginFill(0x808080);
ob.moveTo(-100, -100); ob.lineTo(100, -100); ob.lineTo(100, 40); ob.lineTo(40, 40);
ob.lineTo(40, 100); ob.lineTo(-100, 100); ob.lineTo(-100, -100);
ob.endFill();

// The missile is at (275, 200). Steering puts the shape's point (70, 70), inside
// the gap, under it.
var steerX = 70;
var steerY = 70;
function place() {
    ob._x = 275 - ob._xscale / 100 * steerX;
    ob._y = 200 - ob._yscale / 100 * steerY;
}
ob._xscale = ob._yscale = 20;
place();

var frame = 0;
onEnterFrame = function () {
    frame++;
    ob._xscale = ob._yscale = ob._xscale * 2;
    trace("frame " + frame + ", scale " + ob._xscale + ": hitTest(275, 200, true) = " + ob.hitTest(275, 200, true));
    place();
    if (frame == 5) {
        delete onEnterFrame;
    }
};
