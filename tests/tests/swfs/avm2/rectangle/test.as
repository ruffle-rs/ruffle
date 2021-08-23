package {

	public class Test {
	}
}

import flash.geom.Rectangle;
import flash.geom.Point;

function dump(rect, desc) {
    var extended = "(";
    extended += "top=" + rect.top + ", ";
    extended += "right=" + rect.right + ", ";
    extended += "bottom=" + rect.bottom + ", ";
    extended += "left=" + rect.left + ", ";
    extended += "topLeft=" + rect.topLeft + ", ";
    extended += "bottomRight=" + rect.bottomRight + ", ";
    extended += "width=" + rect.width + ", ";
    extended += "height=" + rect.height + ", ";
    extended += "size=" + rect.size + ", ";
    extended += "x=" + rect.x + ", ";
    extended += "y=" + rect.y + ")";
    trace(" // " + desc);
    trace(extended);
    trace("");
}

function setAndDump(rect, key, value) {
    rect[key] = value;
    dump(rect, "rect." + key + " = " + value);
}

function tryValues(key, values) {
    trace("");
    trace("/// " + key);
    trace("");

    var rect = new Rectangle(1, 3, 5, 7);
    dump(rect, "before modifications");

    for (var i = 0; i < values.length; i++) {
        // Reset, just to make sure we aren't leaking state
        rect = new Rectangle(1, 3, 5, 7);

        setAndDump(rect, key, values[i]);
    }
}

function tryMethod(name, argsList, isRect, dumpOrig) {
    trace("");
    trace("/// " + name);
    trace("");

    var rectList = [new Rectangle(), new Rectangle(1, 3, 5, 7), new Rectangle(-1, -3, 5, 7), new Rectangle(1, 3, -5, 7)];
    for (var h = 0; h < rectList.length; h++) {
        dump(rectList[h], "rect");

        for (var i = 0; i < argsList.length; i++) {
            // Reset, just to make sure we aren't leaking state
            var rect = rectList[h].clone();

            var args = argsList[i];
            var argsText = "";
            for (var j = 0; j < args.length; j++) {
                if (argsText.length > 0) argsText += ", ";
                argsText += args[j];
            }

            var result = rect[name].apply(rect, args);
            if (isRect) {
                dump(result, "rect." + name + "(" + argsText + ")");
            } else {
                trace("// rect." + name + "(" + argsText + ")");
                trace(result);
                trace("");
            }

            if (dumpOrig) dump(rect, "rect");
        }
    }
}

trace("/// Constructor");
trace("");

dump(new Rectangle(), "new Rectangle()");
dump(new Rectangle(1), "new Rectangle(1)");
dump(new Rectangle(1, 2), "new Rectangle(1, 2)");
dump(new Rectangle(1, 2, 3), "new Rectangle(1, 2, 3)");
dump(new Rectangle(1, 2, 3, 4), "new Rectangle(1, 2, 3, 4)");

var numberValues = [0, 100, -200, NaN, Infinity];
tryValues("top", numberValues);
tryValues("right", numberValues);
tryValues("left", numberValues);
tryValues("bottom", numberValues);
tryValues("width", numberValues);
tryValues("height", numberValues);
tryValues("x", numberValues);
tryValues("y", numberValues);

var pointValues = [new Point(0,0), new Point(-100,-200), new Point(100,200), new Point(Infinity,Infinity), new Point(NaN,NaN)];
tryValues("topLeft", pointValues);
tryValues("bottomRight", pointValues);
tryValues("size", pointValues);


trace("");
trace("/// clone");
trace("");

var orig = new Rectangle(1, 3, 5, 7);
var cloned = orig.clone();
dump(orig, "orig");
dump(cloned, "cloned");

trace("// orig == cloned");
trace(orig == cloned);
trace("");

trace("// orig.equals(cloned)");
trace(orig.equals(cloned));
trace("");

trace("");
trace("/// copyFrom");
trace("");

var orig = new Rectangle(1,3,5,7);
var other = new Rectangle(2, 1, 3, 7);
dump(orig,"orig");
dump(other,"other");

trace("// other.copyFrom(orig)");
other.copyFrom(orig);

dump(other,"other");
trace("// orig == other");
trace(orig == other);

trace("");

trace("");
trace("/// equals");
trace("");

var orig = new Rectangle(1,3,5,7);
dump(orig,"orig");

trace("// orig.equals(new Rectangle(1, 3, 5, 7))");
trace(orig.equals(new Rectangle(1, 3, 5, 7)));
trace("");

trace("");
trace("/// isEmpty");
trace("");

trace("// new Rectangle().isEmpty()");
trace(new Rectangle().isEmpty());
trace("");

trace("// new Rectangle(0, 0, 0, 0).isEmpty()");
trace(new Rectangle(0, 0, 0, 0).isEmpty());
trace("");

trace("// new Rectangle(1, 2, 3, 0).isEmpty()");
trace(new Rectangle(1, 2, 3, 0).isEmpty());
trace("");



trace("// new Rectangle(1, 2, 0, 4).isEmpty()");
trace(new Rectangle(1, 2, 0, 4).isEmpty());
trace("");

trace("// new Rectangle(1, 2, 3, 4).isEmpty()");
trace(new Rectangle(1, 2, 3, 4).isEmpty());
trace("");

trace("// new Rectangle(1, 2, Infinity, Infinity).isEmpty()");
trace(new Rectangle(1, 2, Infinity, Infinity).isEmpty());
trace("");

trace("// new Rectangle(1, 2, NaN, NaN).isEmpty()");
trace(new Rectangle(1, 2, NaN, NaN).isEmpty());
trace("");

trace("// new Rectangle(1, 2, undefined, undefined).isEmpty()");
trace(new Rectangle(1, 2, undefined, undefined).isEmpty());
trace("");

trace("// new Rectangle(1, 2, -1, -2).isEmpty()");
trace(new Rectangle(1, 2, -1, -2).isEmpty());
trace("");

trace("");
trace("/// setEmpty");
trace("");

var orig = new Rectangle(1, 3, 5, 7);
dump(orig, "orig");

trace("// orig.setEmpty()");
trace(orig.setEmpty());
trace("");

dump(orig, "orig");


tryMethod("contains", [
//    [],
//    [1],
    [1, 2],
    [1, 3],
    [1.1, 3.1],
    [6, 10],
    [5.9, 9.9],
    [4, NaN],
//    [undefined, 5],
    [5, "5"],
    [5, Infinity],
//    [true],
//    [false],
//    [true, true],
//    [false, false],
//    [true, false],
//    [false, true],
//    [0, 0],
//    [1, 1],
//    [0],
//    [1],
//    [{}],
//    [{}, {}],
//    [Infinity],
//    [NaN],
//    [new Point(1, 3)],
//    []
], false, false);

tryMethod("containsPoint", [
    [new Point()],
    [new Point(1)],
    [new Point(1, 2)],
    [new Point(1, 3)],
    [new Point(1.1, 3.1)],
    [new Point(6, 10)],
    [new Point(5.9, 9.9)],
//    [new Point(undefined, 5)],
//    [{x: 5, y: Infinity}],
//    [{x: 5, y: 5}]
], false, false);

tryMethod("containsRect", [
    [new Rectangle(0.9, 2.9, 5, 7)],
    [new Rectangle(1, 3, 5.1, 7.1)],
    [new Rectangle(5, 5, NaN, 1)]
], false, false);

tryMethod("inflate", [
//    [],
//    [1],
    [1, 2],
    [-3, -4],
    [Infinity, 5],
    [5, NaN],
//    [3, {}]
], false, true);

tryMethod("inflatePoint", [
//    [],
//    [new Point()],
    [new Point(1, 2)],
//    [{x: 3, y: 4}],
//    [{x: 5}]
], false, true);

tryMethod("intersection", [
//    [],
//    [new Rectangle(1, 3, 5, 7)],
    [new Rectangle(3, 5, 7, 9)],
    [new Rectangle(-1, -3, 5, 7)],
    [new Rectangle(30, 50, 1, 1)],
//    [{x: 3, y: 5, width: 7, height: 1}],
//    [{x: 3, y: 5, width: 2}]
], true, false);

tryMethod("intersects", [
//    [],
//    [new Rectangle(1, 3, 5, 7)],
    [new Rectangle(3, 5, 7, 9)],
    [new Rectangle(-1, -3, 5, 7)],
//    [{x: 3, y: 5, width: 7, height: 1}],
//    [{x: 3, y: 5, width: 2}]
], false, false);

tryMethod("offset", [
//    [],
//    [1],
    [1, 2],
    [-3, -4],
    [Infinity, 5],
    [NaN, 6],
//    [5, NaN],
//    [3, {}]
], false, true);

tryMethod("offsetPoint", [
//    [],
//    [new Point()],
    [new Point(1, 2)],
//    [{x: 3, y: 4}],
//    [{x: 5}]
], false, true);

tryMethod("union", [
//    [],
//    [new Rectangle(1, 3, 5, 7)],
    [new Rectangle(3, 5, 7, 9)],
    [new Rectangle(3, 5, 7, 9)],
    [new Rectangle(-1, -3, NaN, 7)],
//    [{x: 3, y: 5, width: 7, height: 1}],
//    [{x: 3, y: 5, width: 2}]
], true, false);

tryMethod("setTo", [
    [3,5,7,9],
    [-1,-3,5,7]
], false, true);
