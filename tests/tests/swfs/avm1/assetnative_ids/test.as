// Check which IDs are assigned to methods.
// We'll use Date for that, as in playerglobals it's defined as:
//
//     Date = ASconstructor(103,256);
//     var o = Date.prototype;
//     ASSetNative(o,103,"getFullYear,getYear,getMonth,getDate,getDay,getHours,getMinutes,getSeconds,getMilliseconds,setFullYear,setMonth,setDate,setHours,setMinutes,setSeconds,setMilliseconds,getTime,setTime,getTimezoneOffset,toString,setYear");
//
// In the case above, getTime should have ID 16.

var d = new Date(1234);

// Make sure that getTime is 16.
d.id16 = ASnative(103, 16);

trace("d.getTime = " + d.getTime);
trace("d.getTime() = " + d.getTime());
trace("d.id16 = " + d.id16);
trace("d.id16() = " + d.id16());

// Now use ASSetNative to set the method.
ASSetNative(
    d, 103,
    "getTime2",
    16
);

trace("d.getTime2() = " + d.getTime2());

// What about empty names?
ASSetNative(
    d, 103,
    ",getTime3",
    15
);

trace("d.getTime3() = " + d.getTime3());

// Negative values?
ASSetNative(
    d, 103,
    ",,,,,,,,,,,,,,,,,getTime4",
    -1
);

trace("d.getTime4() = " + d.getTime4());

// Noisy values?
var m = {};
m.valueOf = function() { trace("valueOf"); return 16; };
ASSetNative(
    d, 103,
    "getTime5",
    m
);

trace("d.getTime5() = " + d.getTime5());

// Exceptions
var m = {};
m.valueOf = function() { throw "some error"; };
try {
    ASSetNative(
        d, 103,
        "getTime6",
        m
    );
} catch (err) {
    trace("Caught: " + err);
}
