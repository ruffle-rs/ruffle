import flash.display.BitmapData;
import flash.geom.Rectangle;

var disposedBmd;
var validBmd;
var triedSoFar = [];
var objLooksLikeNum = {valueOf: function() { return 2; }};

function main() {
    var createTransparent = function() { return new BitmapData(3, 3, true, 0xAABBCCDD); };
    var createOpaque = function() { return new BitmapData(3, 3, false, 0xAABBCCDD); };
    var createDisposed = function() { var o = createTransparent(); o.dispose(); return o; };

    // dumpBmdAfter, name, [args...]
    var functionsAndArgs = [
        [false, "getPixel32", 1, 2]
    ];

    var objects = [["Transparent", createTransparent], ["Opaque", createOpaque], ["Disposed", createDisposed]];

    for (var o = 0; o < objects.length; o++) {
        triedSoFar = [];
        trace("-- " + objects[o][0] + " --");
        for (var i = 0; i < functionsAndArgs.length; i++) {
            var dump = functionsAndArgs[i][0];
            var callback = function(bmd) {
                if (dump) {
                    printBmd(bmd);
                }
            };
            callWithDifferentArgs(objects[o][1], callback, functionsAndArgs[i][1], functionsAndArgs[i].slice(2));
        }
        trace("");
        trace("");
    }
}

function generateBadArguments(good) {
    var result = [null, undefined, good, {}];
    result.push(objLooksLikeNum);
    result.push(10);
    result.push(-2);
    return result;
}

function callWithDifferentArgs(createObject, callback, functionName, knownGoodArguments) {
    var allArgsToTest = generateArgSets(knownGoodArguments);
    for (var i = 0; i < allArgsToTest.length; i++) {
        var argStr = "";
        for (var j = 0; j < allArgsToTest[i].length; j++) {
            if (j > 0) {
                argStr += ", ";
            }
            argStr += valueToString(allArgsToTest[i][j]);
        }
        trace("// " + functionName + "(" + argStr + ")");
        var object = createObject();
        var f = object[functionName].apply(object, allArgsToTest[i]);
        trace(valueToString(f));
        callback(object);
        trace("");
    }
}

function generateArgSets(knownGoodArguments) {
    var results = [];

    // Partial sets (e.g. [], [1], [1, 2], ...)
    for (var i = 0; i <= knownGoodArguments.length; i++) {
        results.push(knownGoodArguments.slice(0, i));
    }

    // Replace one argument at a time with each of its specific bad variants
    for (var i = 0; i < knownGoodArguments.length; i++) {
        var bads = generateBadArguments(knownGoodArguments[i]);
        for (var b = 0; b < bads.length; b++) {
            var variant = knownGoodArguments.concat();
            variant[i] = bads[b];
            results.push(variant);
        }
    }


    // Deduplicate
    var unique = [];
    for (var i = 0; i < results.length; i++) {
        var a = results[i];
        var found = false;
        for (var j = 0; j < triedSoFar.length; j++) {
            if (arraysEqual(a, triedSoFar[j])) { found = true; break; }
        }
        if (!found) {
            triedSoFar.push(a);
            unique.push(a);
        }
    }

    return unique;
}

function arraysEqual(a, b) {
    if (a.length !== b.length) return false;
    for (var i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}

function valueToString(v) {
    if (v === objLooksLikeNum) {
        return "objLooksLikeNum";
    }
    if (v instanceof Array) {
        var result = "";
        for (var i = 0; i < result.length; i++) {
            if (i > 0) {
                result += ", ";
            }
            result += valueToString(v);
        }
        return result;
    }
    if (typeof v == "string") {
        var result = "";
        for (var i= 0; i < v.length; i++) {
            var c = v.charAt(i);
            if (c == "\\") result += "\\\\";
            else if (c == "\"") result += "\\\"";
            else if (c == "\n") result += "\\n";
            else if (c == "\r") result += "\\r";
            else if (c == "\t") result += "\\t";
            else result += c;
        }
        return "\"" + result + "\"";
    }
    if (typeof v == "object") {
        var props = [];
        if (v instanceof Rectangle) {
            props.push("width");
            props.push("height");
            props.push("x");
            props.push("y");
        } else {
            for (var prop in v) {
                if (typeof v[prop] !== "function") {
                    props.push(prop);
                }
            }
        }
        props.sort();
        var str = "";
        for (var i = 0; i < props.length; i++) {
            var prop = props[i];
            if (str != "") str += ", ";
            str += prop + "=" + valueToString(v[prop]);
        }
        if (props.length == 0) return "{}";
        return "{ " + str + " }";
    }
    return "" + v;
}

function hex(bmd, x, y) {
    var color = bmd.getPixel32(x, y);
    var alpha:String = (color >> 24 & 0xFF).toString(16);
    var red:String = (color >> 16 & 0xFF).toString(16);
    var green:String = (color >> 8 & 0xFF).toString(16);
    var blue:String = (color & 0xFF).toString(16);
    return "0x" + alpha + red + green + blue;
}

function printBmd(bmd) {
    trace("--");
    for (var y = 0; y < bmd.height; y++) {
        var row = "";
        for (var x = 0; x < bmd.width; x++) {
            if (x > 0) {
                row += " ";
            }
            row += hex(bmd, x, y);
        }
        trace(row);
    }
    trace("--");
}



main();