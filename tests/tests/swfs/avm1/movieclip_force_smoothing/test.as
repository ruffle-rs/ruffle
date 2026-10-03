import flash.display.BitmapData;
import flash.geom.Matrix;

// Every clip scales what it holds up to 100x100. The first row loads image.png, a 2x2 image.
// The second row loads the SWFs made by generate.py, which hold a 2x2 shape with a bitmap fill
// that is not smoothed, except for solid.swf, whose shape has a solid fill.
var loader:MovieClipLoader = new MovieClipLoader();
var listener:Object = new Object();
var queue:Array = [];
var row:Number = 0;
var column:Number = 0;

function makeClip(name:String):MovieClip {
    var clip:MovieClip = createEmptyMovieClip(name, getNextHighestDepth());
    clip._x = 110 * column++;
    clip._y = 110 * row;
    clip._xscale = clip._yscale = 5000;
    return clip;
}

// The movies are loaded one after another, to keep the traces in order.
function load(clip:MovieClip, url:String, loaded:Function):Void {
    queue.push({clip: clip, url: url, loaded: loaded});
}

function loadNext():Void {
    if (queue.length > 0) {
        loader.loadClip(queue[0].url, queue[0].clip);
    }
}

listener.onLoadInit = function(clip:MovieClip):Void {
    queue.shift().loaded(clip);
    trace(clip._name + ": " + clip.forceSmoothing);
    loadNext();
};
loader.addListener(listener);

function untouched(clip:MovieClip):Void {
}

function enable(clip:MovieClip):Void {
    clip.forceSmoothing = true;
}

function enableThenDisable(clip:MovieClip):Void {
    clip.forceSmoothing = true;
    clip.forceSmoothing = false;
}

var bitmap:BitmapData = new BitmapData(2, 2, false, 0xFF0000);
bitmap.setPixel(1, 0, 0x00FF00);
bitmap.setPixel(0, 1, 0x0000FF);
bitmap.setPixel(1, 1, 0xFFFF00);

load(makeClip("image_untouched"), "image.png", untouched);
load(makeClip("image_enabled"), "image.png", enable);
load(makeClip("image_disabled"), "image.png", enableThenDisable);

// Loading the image discards what was set before.
var early:MovieClip = makeClip("image_early");
early.forceSmoothing = true;
load(early, "image.png", untouched);

// Loading another image as well.
var reloaded:MovieClip = makeClip("image_reloaded");
load(reloaded, "image.png", enable);
load(reloaded, "image.png", untouched);

// Only the clip that the image was loaded into can smooth it.
var outer:MovieClip = makeClip("image_outer");
load(outer.createEmptyMovieClip("inner", 0), "image.png", function(clip:MovieClip):Void {
    outer.forceSmoothing = true;
});

// Attached bitmaps only follow the `smoothing` argument of `attachBitmap`.
var attached:MovieClip = makeClip("image_attached");
attached.attachBitmap(bitmap, 0, "auto", false);
attached.forceSmoothing = true;
trace(attached._name + ": " + attached.forceSmoothing);

row = 1;
column = 0;

load(makeClip("shape_untouched"), "filled.swf", untouched);
load(makeClip("shape_enabled"), "filled.swf", enable);
load(makeClip("shape_disabled"), "filled.swf", enableThenDisable);

// Loading another movie discards what was set before.
var reloadedShape:MovieClip = makeClip("shape_reloaded");
load(reloadedShape, "filled.swf", enable);
load(reloadedShape, "filled.swf", untouched);

// Only the clip that the shape is placed in can smooth it, loaded or not.
load(makeClip("shape_outer"), "nested.swf", enable);
load(makeClip("shape_inner"), "nested.swf", function(clip:MovieClip):Void {
    clip.child.forceSmoothing = true;
});

// Shapes that are placed later on are smoothed as well.
load(makeClip("shape_later"), "frames.swf", function(clip:MovieClip):Void {
    clip.forceSmoothing = true;
    trace("set on frame " + clip._currentframe);
});

// Shapes without bitmap fills stay the same.
load(makeClip("shape_solid"), "solid.swf", enable);

// Drawings are not shapes.
var drawn:MovieClip = makeClip("shape_drawn");
drawn.beginBitmapFill(bitmap, new Matrix(), false, false);
drawn.lineTo(2, 0);
drawn.lineTo(2, 2);
drawn.lineTo(0, 2);
drawn.endFill();
drawn.forceSmoothing = true;

loadNext();
