// Check which IDs are assigned to properties.
// We'll use flash.filters.ConvolutionFilter for that, as in playerglobals it's
// defined as:
//
//     flash.filters.ConvolutionFilter = ASconstructor(1109,0);
//     flash.filters.ConvolutionFilter.prototype = o = new flash.filters.BitmapFilter();
//     ASSetNativeAccessor(o,1109,"8matrixX,8matrixY,8matrix,8divisor,8bias,8preserveAlpha,8clamp,8color,8alpha",1);
//
// In the case above, alpha getter should have ID 17, whereas alpha setter 18.

var f = new flash.filters.ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 0, true, false, 0x0000FF, 1.0);

// Make sure that getter and setter are 17 and 18.
f.id17 = ASnative(1109, 17);
f.id18 = ASnative(1109, 18);

trace("f.id17() = " + f.id17());
f.id18(0.8);
trace("f.alpha = " + f.alpha);
trace("f.id17() = " + f.id17());

// Now use ASSetNativeAccessor to set the custom property.
ASSetNativeAccessor(
    f, 1109,
    "alphaTwo",
    17
);

// Check that it behaves the same way as alpha.
trace("f.alpha = " + f.alpha);
trace("f.alphaTwo = " + f.alphaTwo);
f.alphaTwo = 0.2;
trace("f.alpha = " + f.alpha);
trace("f.alphaTwo = " + f.alphaTwo);

// Now check how empty properties behave.
ASSetNativeAccessor(
    f, 1109,
    ",alphaThree",
    15
);

trace("f.alpha = " + f.alpha);
trace("f.alphaThree = " + f.alphaThree);
f.alphaThree = 0.8;
trace("f.alpha = " + f.alpha);
trace("f.alphaThree = " + f.alphaThree);

// Negative values?
ASSetNativeAccessor(
    f, 1109,
    ",,,,,,,,,alphaFour",
    -1
);

trace("f.alpha = " + f.alpha);
trace("f.alphaFour = " + f.alphaFour);
f.alphaFour = 0.2;
trace("f.alpha = " + f.alpha);
trace("f.alphaFour = " + f.alphaFour);

// Noisy values?
var m = {};
m.valueOf = function() { trace("valueOf"); return 17; };
ASSetNativeAccessor(
    f, 1109,
    "alphaFive",
    m
);

trace("f.alpha = " + f.alpha);
trace("f.alphaFive = " + f.alphaFive);
f.alphaFive = 0.6;
trace("f.alpha = " + f.alpha);
trace("f.alphaFive = " + f.alphaFive);

// Exceptions
var m = {};
m.valueOf = function() { throw "some error"; };
try {
    ASSetNativeAccessor(
        f, 1109,
        "alphaFive",
        m
    );
} catch (err) {
    trace("Caught: " + err);
}
