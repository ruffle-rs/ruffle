function Microphone() {
    // empty
};

// This wrapper immediately delegates to the corresponding native method.
// Observable as a native method would have no prototype.
Microphone["get"] = function(index) {
    // Doesn't work because of a Rascal bug: https://github.com/Dinnerbone/Rascal/issues/71
    // TODO: Remove the extra variable once the bug is fixed
    // return ASnative(2104, 200)(index);

    var f = ASnative(2104, 200);
    return f(index);
};

Microphone.addProperty("names", ASnative(2104, 201), null);

var o = Microphone.prototype;

// Native IDs:         0                1        2        3                       4          5                    6
ASSetNative(o, 2104, "6setSilenceLevel,6setRate,6setGain,6setUseEchoSuppression,10setCodec,10setFramesPerPacket,10setEncodeQuality");

// DONT_ENUM, DONT_DELETE
ASSetPropFlags(o, null, 3);
