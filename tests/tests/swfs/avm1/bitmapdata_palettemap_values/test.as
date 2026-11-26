// Checks how BitmapData.paletteMap combines the values of its channel
// arrays: values wider than one channel, sums that overflow, negative,
// fractional and string values, short arrays, holes, arguments that aren't
// arrays and omitted arrays.

import flash.display.BitmapData;
import flash.geom.Point;
import flash.geom.Rectangle;

function hex(n) {
    var digits = "0123456789ABCDEF";
    var s = "";
    for (var i = 7; i >= 0; i--) {
        s += digits.charAt((n >> (i * 4)) & 0xF);
    }
    return "0x" + s;
}

function filled(value) {
    var a = [];
    for (var i = 0; i < 256; i++) {
        a.push(value);
    }
    return a;
}

function identity(shift) {
    var a = [];
    for (var i = 0; i < 256; i++) {
        a.push(i << shift);
    }
    return a;
}

var source = new BitmapData(4, 1, true, 0);
source.setPixel32(0, 0, 0xFF000000);
source.setPixel32(1, 0, 0xFF010203);
source.setPixel32(2, 0, 0xFF808080);
source.setPixel32(3, 0, 0xFFFFFFFF);

function run(name, r, g, b, a) {
    var dest = new BitmapData(4, 1, true, 0x12345678);
    dest.paletteMap(source, new Rectangle(0, 0, 4, 1), new Point(0, 0), r, g, b, a);
    var out = [];
    for (var x = 0; x < 4; x++) {
        out.push(hex(dest.getPixel32(x, 0)));
    }
    trace(name + ": " + out.join(" "));
    dest.dispose();
}

run("identity arrays", identity(16), identity(8), identity(0), identity(24));
run("red 0x00110000", filled(0x00110000), identity(8), identity(0), identity(24));
run("red 0xFF000000 only", filled(0xFF000000), filled(0), filled(0), filled(0));
run("green 0x00FF00FF", filled(0), filled(0x00FF00FF), filled(0), filled(0xFF000000));
run("blue 0x01020304", filled(0), filled(0), filled(0x01020304), filled(0));
run("sum overflow", filled(0x80808080), filled(0x80808080), filled(0), filled(0));
run("all ones", filled(0xFFFFFFFF), filled(0xFFFFFFFF), filled(0), filled(0));
run("negative", filled(-1), filled(0), filled(0), filled(0));
run("fractional", filled(1.9), filled(0), filled(0), filled(0xFF000000));
run("strings", filled("0x10"), filled("abc"), filled(0), filled(0xFF000000));

var shortArray = [0x00AA0000, 0x00BB0000];
run("short red", shortArray, identity(8), identity(0), identity(24));

var holes = identity(16);
delete holes[1];
holes[128] = undefined;
holes[255] = null;
run("red holes", holes, identity(8), identity(0), identity(24));

run("red null", null, identity(8), identity(0), identity(24));
run("red undefined", undefined, identity(8), identity(0), identity(24));
run("red number", 5, identity(8), identity(0), identity(24));
run("red object", {}, identity(8), identity(0), identity(24));
run("all omitted", undefined, undefined, undefined, undefined);

var custom = {length: 256};
for (var i = 0; i < 256; i++) {
    custom[i] = 0x00010000 * (255 - i);
}
run("red plain object with indices", custom, identity(8), identity(0), identity(24));

trace("Done");
