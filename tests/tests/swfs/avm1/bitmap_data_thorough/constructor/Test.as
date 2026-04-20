import flash.display.BitmapData;
import flash.geom.Rectangle;

var disposedBmd;
var validBmd;
var triedSoFar = [];
var objLooksLikeNum = {valueOf: function() { return 2; }};

function main() {
    var printBmd = function(bmd) {
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
    constructWithDifferentArgs("BitmapData", printBmd, 5, 6, true, 0xAABBCCDD);
    constructWithDifferentArgs("BitmapData", printBmd, 3, 4, false, 0x12345678);
}

function generateBadArguments(good) {
    var result = [null, undefined, good, {}];
    result.push(objLooksLikeNum);
    return result;
}

function constructWithDifferentArgs(className, callback) {
    var allArgsToTest = generateArgSets(arguments.slice(2));
    for (var i = 0; i < allArgsToTest.length; i++) {
        var argStr = "";
        for (var j = 0; j < allArgsToTest[i].length; j++) {
            if (j > 0) {
                argStr += ", ";
            }
            argStr += valueToString(allArgsToTest[i][j]);
        }
        trace("// new " + className + "(" + argStr + ")");
        var clas = _global.flash.display[className];
        var f = constructWithArgs(clas, allArgsToTest[i]);
        trace(valueToString(f));
        if (f instanceof clas) {
            callback(f);
        } else {
            trace("NOT AN INSTANCEOF!");
        }
        trace("");
    }
}

function constructWithArgs(cls, args) {
    switch (args.length) {
        case 0: return new cls();
        case 1: return new cls(args[0]);
        case 2: return new cls(args[0], args[1]);
        case 3: return new cls(args[0], args[1], args[2]);
        case 4: return new cls(args[0], args[1], args[2], args[3]);
        case 5: return new cls(args[0], args[1], args[2], args[3], args[4]);
        case 6: return new cls(args[0], args[1], args[2], args[3], args[4], args[5]);
        case 7: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6]);
        case 8: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7]);
        case 9: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8]);
        case 10: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9]);
        case 11: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10]);
        case 12: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11]);
        case 13: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11], args[12]);
        case 14: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11], args[12], args[13]);
        case 15: return new cls(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8], args[9], args[10], args[11], args[12], args[13], args[14]);
    }
    trace("INVALID TEST: Too many arguments!");
    return null;
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



main();