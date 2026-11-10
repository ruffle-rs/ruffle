function Camera() {
    // empty
};

// This wrapper immediately delegates to the corresponding native method.
// Observable as a native method would have no prototype.
Camera["get"] = function(index) {
    // Doesn't work because of a Rascal bug: https://github.com/Dinnerbone/Rascal/issues/71
    // TODO: Remove the extra variable once the bug is fixed
    // return ASnative(2102, 200)(index);

    var f = ASnative(2102, 200);
    return f(index);
};

Camera.addProperty("names", ASnative(2102, 201), null);

var o = Camera.prototype;

// Native IDs:         0        1           2                    3               4            5
ASSetNative(o, 2102, "6setMode,6setQuality,6setKeyFrameInterval,6setMotionLevel,6setLoopback,6setCursor");

// DONT_ENUM, DONT_DELETE
ASSetPropFlags(o, null, 3);
