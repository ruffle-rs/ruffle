import flash.display.BitmapData;

this.createEmptyMovieClip("img_mc", this.getNextHighestDepth());
trace("img_mc SWF version:");
trace(img_mc.getSWFVersion());
var mcl:MovieClipLoader = new MovieClipLoader();
mcl.loadClip("dots.png", img_mc);
trace("img_mc SWF version immediately after load:");
trace(img_mc.getSWFVersion());
img_mc._x = 160;

this.createEmptyMovieClip("bmd_mc", this.getNextHighestDepth());
var bmd:BitmapData = new BitmapData(128, 128, true, 0xFF00FF00);
bmd_mc.attachBitmap(bmd, bmd_mc.getNextHighestDepth());
bmd_mc._x = 160;
bmd_mc._y = 160;

function traceRounded(value) {
    trace(Math.round(value * 1000) / 1000);
}

function traceMatrixRounded(mat) {
    traceRounded(mat.a);
    traceRounded(mat.b);
    traceRounded(mat.c);
    traceRounded(mat.d);
    traceRounded(mat.tx);
    traceRounded(mat.ty);
}

function flip(mc) {
    trace("setting _xscale to -100 on " + mc._name);
    mc._xscale = -100;
    trace("_xscale:");
    traceRounded(mc._xscale);
    trace("_yscale:");
    traceRounded(mc._yscale);
    trace("_rotation:");
    traceRounded(mc._rotation);
    trace("matrix:");
    traceMatrixRounded(mc.transform.matrix);
    trace("SWF version:");
    trace(mc.getSWFVersion());
    trace("---");
}

// bmd_mc is slightly later to make their order in the trace output guaranteed
setTimeout(function() { flip(img_mc); }, 1000);
setTimeout(function() { flip(bmd_mc); }, 1100);

setTimeout(function() { flip(img_mc); }, 2000);
setTimeout(function() { flip(bmd_mc); }, 2100);

setTimeout(function() { flip(img_mc); }, 3000);
setTimeout(function() { flip(bmd_mc); }, 3100);

function done() {
    fscommand("quit");
}

setTimeout(done, 4000);