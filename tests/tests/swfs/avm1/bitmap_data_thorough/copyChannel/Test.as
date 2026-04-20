import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.geom.Point;

var disposedBmd;
var transparentBmd;
var opaqueBmd;
var triedSoFar = [];
var objLooksLikeNum = {valueOf: function() { return 2; }};
var rect = new Rectangle(0, 0, 2, 2);
var rectAsObj = {x: 0, y: 0, width: 2, height: 2};
var rectWithoutWidth = {x: 0, y: 0, height: 2};
var zeroSizedRect = new Rectangle(1, 1, 0, 0);
var point = new Point(1, 1);
var pointOutsideBmd = new Point(100, 100);
var pointAsObj = {x: 0, y: 0};
var pointWithoutY = {x: 0};

function main() {
    var createTransparent = function() { return new BitmapData(3, 3, true, 0xAABBCCDD); };
    var createOpaque = function() { return new BitmapData(3, 3, false, 0xAABBCCDD); };
    var createDisposed = function() { var o = createTransparent(); o.dispose(); return o; };

    disposedBmd = createDisposed();
    opaqueBmd = new BitmapData(3, 3, false, 0x12345678);
    transparentBmd = new BitmapData(3, 3, true, 0x12345678);

    // dumpBmdAfter, name, [args...]
    var functionsAndArgs = [
        [true, "copyChannel", transparentBmd, rect, point, 2, 4],
        [true, "copyChannel", transparentBmd, rect, point, 3, 1],
        [true, "copyChannel", transparentBmd, rect, point, 8, 2],
        [true, "copyChannel", transparentBmd, rect, point, 2, 3]
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
    if (typeof good == "number") {
        result.push(objLooksLikeNum);
        result.push(-1);
        result.push(-2);
        result.push(10);
    }
    if (good === opaqueBmd) {
        result.push(transparentBmd);
        result.push(disposedBmd);
    }
    if (good === transparentBmd) {
        result.push(opaqueBmd);
        result.push(disposedBmd);
    }
    if (good == rect) {
        result.push(zeroSizedRect);
        result.push(rectAsObj);
        result.push(rectWithoutWidth);
    }
    if (good == point) {
        result.push(pointOutsideBmd);
        result.push(pointAsObj);
        result.push(pointWithoutY);
    }

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
    if (v === transparentBmd) {
        return "transparentBmd";
    }
    if (v === opaqueBmd) {
        return "opaqueBmd";
    }
    if (v === disposedBmd) {
        return "disposedBmd";
    }
    if (v === rect) {
        return "rect";
    }
    if (v === zeroSizedRect) {
        return "zeroSizedRect";
    }
    if (v === rectAsObj) {
        return "rectAsObj";
    }
    if (v === rectWithoutWidth) {
        return "rectWithoutWidth";
    }
    if (v === point) {
        return "point";
    }
    if (v === pointOutsideBmd) {
        return "pointOutsideBmd";
    }
    if (v === pointAsObj) {
        return "pointAsObj";
    }
    if (v === pointWithoutY) {
        return "pointWithoutY";
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
    if (alpha.length == 1) {
        alpha = "0" + alpha;
    }
    var red:String = (color >> 16 & 0xFF).toString(16);
    if (red.length == 1) {
        red = "0" + red;
    }
    var green:String = (color >> 8 & 0xFF).toString(16);
    if (green.length == 1) {
        green = "0" + green;
    }
    var blue:String = (color & 0xFF).toString(16);
    if (blue.length == 1) {
        blue = "0" + blue;
    }
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