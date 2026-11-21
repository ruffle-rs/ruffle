// A clip with a filter and alpha (like a shadow) is drawn through a cached
// bitmap: blurred, and with its overlapping parts sharing one alpha. An empty
// child with a filter of its own (like a muzzle flash on an empty frame) must
// not change that. On the right is the same clip without the empty child.

function rect(clip, depth, x, y, w, h) {
    var part = clip.createEmptyMovieClip("part" + depth, depth);
    part.beginFill(0x000000);
    part.moveTo(x, y);
    part.lineTo(x + w, y);
    part.lineTo(x + w, y + h);
    part.lineTo(x, y + h);
    part.lineTo(x, y);
    part.endFill();
}

function shadow(name, depth, x, withEmptyChild) {
    var clip = _root.createEmptyMovieClip(name, depth);
    clip._x = x;
    clip._y = 40;
    clip._xscale = 200;
    clip._yscale = 200;
    rect(clip, 1, 10, 10, 50, 30);
    rect(clip, 2, 30, 20, 40, 40);
    if (withEmptyChild) {
        var muzzle = clip.createEmptyMovieClip("muzzle", 3);
        muzzle.filters = [new flash.filters.GlowFilter(0xFFFF00, 1, 8, 8)];
    }
    clip._alpha = 40;
    clip.filters = [new flash.filters.BlurFilter(4, 4, 1)];
}

shadow("withEmptyChild", 1, 60, true);
shadow("withoutEmptyChild", 2, 320, false);
