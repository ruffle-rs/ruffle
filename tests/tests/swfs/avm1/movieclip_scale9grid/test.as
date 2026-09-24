// scale9Grid on AVM1: any rectangle is accepted, even a zero-sized one or one reaching
// outside the bounds; a negative width or height clears the grid instead of throwing.

var clip = _root.createEmptyMovieClip("clip", 1);
clip.beginFill(0xFF0000);
clip.moveTo(0, 0);
clip.lineTo(60, 0);
clip.lineTo(60, 60);
clip.lineTo(0, 60);
clip.endFill();

function show(name, rect) {
    clip.scale9Grid = rect;
    var got = clip.scale9Grid;
    if (got == undefined) {
        trace(name + ": undefined");
    } else {
        trace(name + ": " + got.x + "," + got.y + "," + got.width + "," + got.height);
    }
}

trace("initial: " + clip.scale9Grid);

show("inside", new flash.geom.Rectangle(20, 20, 20, 20));
show("equal to bounds", new flash.geom.Rectangle(0, 0, 60, 60));
show("outside", new flash.geom.Rectangle(10, 10, 100, 100));
show("zero size", new flash.geom.Rectangle(20, 20, 0, 0));
show("negative width", new flash.geom.Rectangle(20, 20, -10, 20));
show("negative height", new flash.geom.Rectangle(20, 20, 20, -10));
show("fractional", new flash.geom.Rectangle(10.99, 10.99, 20.01, 20.01));

clip.scale9Grid = new flash.geom.Rectangle(20, 20, 20, 20);
clip.scale9Grid = null;
trace("cleared with null: " + clip.scale9Grid);

clip.scale9Grid = new flash.geom.Rectangle(20, 20, 20, 20);
clip.scale9Grid = undefined;
trace("cleared with undefined: " + clip.scale9Grid);
