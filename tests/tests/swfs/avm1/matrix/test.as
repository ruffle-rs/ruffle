import flash.geom.Matrix;
import flash.geom.Point;

function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
}

function addTracingProperty(name, obj, prop, value) {
    obj.addProperty(prop, function() {
        trace("  get " + name + "." + prop);
        return value;
    }, function(newValue) {
        trace("  set " + name + "." + prop + " = " + newValue);
        value = newValue;
    });
}

function tracingMethod(name, obj, method) {
    var original = obj[method];
    obj[method] = function() {
        var args = "";
        for (var i = 0; i < arguments.length; i++) {
            if (i > 0) {
                args += ", ";
            }
            args += arguments[i];
        }
        trace("  call " + name + "." + method + "(" + args + ")");
        return original.apply(this, arguments);
    };
}

function tracingNumber(name, value) {
    return {valueOf: function() {
        trace("  valueOf " + name);
        return value;
    }};
}

function tracingMatrix(name, a, b, c, d, tx, ty) {
    var matrix = new Matrix();
    addTracingProperty(name, matrix, "a", a);
    addTracingProperty(name, matrix, "b", b);
    addTracingProperty(name, matrix, "c", c);
    addTracingProperty(name, matrix, "d", d);
    addTracingProperty(name, matrix, "tx", tx);
    addTracingProperty(name, matrix, "ty", ty);
    return matrix;
}

function tracingPoint(name, x, y) {
    var point = new Point();
    addTracingProperty(name, point, "x", x);
    addTracingProperty(name, point, "y", y);
    return point;
}

function tracingMatrixWithMethods(name, a, b, c, d, tx, ty) {
    var matrix = tracingMatrix(name, a, b, c, d, tx, ty);
    tracingMethod(name, matrix, "identity");
    tracingMethod(name, matrix, "concat");
    tracingMethod(name, matrix, "rotate");
    tracingMethod(name, matrix, "scale");
    tracingMethod(name, matrix, "createBox");
    tracingMethod(name, matrix, "deltaTransformPoint");
    return matrix;
}

var matrix;
var other;
var i;
var result;

trace("/// Constructors");

trace("// new Matrix()");
trace(new Matrix());
trace("");

trace("// new Matrix(1)");
trace(new Matrix(1));
trace("");

trace("// new Matrix(1, 2, 3, {})");
trace(new Matrix(1, 2, 3, {}));
trace("");

trace("// new Matrix(1, 2, 3, 4, 5, 6)");
trace(new Matrix(1, 2, 3, 4, 5, 6));
trace("");

trace("// new Matrix(1, 2, 3, 4, 5, 6, 7)");
trace(new Matrix(1, 2, 3, 4, 5, 6, 7));
trace("");

trace("// new Matrix(undefined)");
trace(new Matrix(undefined));
trace("");

trace("// new Matrix('1', '2', '3', '4', '5', '6')");
trace(new Matrix("1", "2", "3", "4", "5", "6"));
trace("");
trace("");

trace("/// Property enumeration order");

trace("// new Matrix()");
trace(keys(new Matrix()));
trace("");

trace("// new Matrix(1, 2, 3, 4, 5, 6)");
trace(keys(new Matrix(1, 2, 3, 4, 5, 6)));
trace("");

trace("// new Matrix(1, 2, 3, 4, 5, 6).clone()");
trace(keys((new Matrix(1, 2, 3, 4, 5, 6)).clone()));
trace("");

trace("// {} after identity()");
matrix = {};
Matrix.prototype.identity.call(matrix);
trace(keys(matrix));
trace("");

trace("// {} after concat(new Matrix())");
matrix = {};
Matrix.prototype.concat.call(matrix, new Matrix());
trace(keys(matrix));
trace("");

trace("// {a: 2, d: 4} after invert()");
matrix = {a: 2, d: 4};
Matrix.prototype.invert.call(matrix);
trace(keys(matrix));
trace("");

trace("// Matrix.prototype");
trace(keys(Matrix.prototype));
trace("");
trace("");

trace("/// identity");

matrix = new Matrix(1, 2, 3, 4, 5, 6);
trace("// matrix.identity()");
trace(matrix.identity());
trace(matrix);
trace("");

trace("// matrix.identity()");
matrix = tracingMatrix("matrix", 1, 2, 3, 4, 5, 6);
matrix.identity();
trace("");
trace("");

trace("/// clone");

matrix = new Matrix(1, 2, 3, 4, 5, 6);
trace("// matrix.clone()");
trace(matrix.clone());
trace("");

trace("// matrix.clone() === matrix");
trace(matrix.clone() === matrix);
trace("");

trace("// Matrix.prototype.clone.call({a: 1, b: 2, c: 3, d: 4, tx: 5, ty: 6})");
var clone = Matrix.prototype.clone.call({a: 1, b: 2, c: 3, d: 4, tx: 5, ty: 6});
trace(clone);
trace(clone instanceof Matrix);
trace("");

trace("// matrix.clone()");
matrix = tracingMatrix("matrix", 1, 2, 3, 4, 5, 6);
trace(matrix.clone());
trace("");
trace("");

trace("/// concat");

var concatCases = [
    [new Matrix(), new Matrix(11, 13, 17, 19, 23, 29)],
    [new Matrix(11, 13, 17, 19, 23, 29), new Matrix(3, 5, 0, 5, 0, 5)],
    [new Matrix(2, 0, 0, 3, 5, 7), new Matrix(11, 0, 0, 13, 17, 19)],
    [new Matrix(2, 0, 0, 3, 5, 7), new Matrix(11, 1, 0, 13, 17, 19)],
    [new Matrix(2, 0, 1, 3, 5, 7), new Matrix(11, 0, 0, 13, 17, 19)],
    [new Matrix(2, NaN, 0, 3, 5, 7), new Matrix(11, 0, 0, 13, 17, 19)],
    [new Matrix(2, 0, 0, 3, "5", "7"), new Matrix(11, 0, 0, 13, "17", "19")]
];
for (i = 0; i < concatCases.length; i++) {
    matrix = concatCases[i][0];
    other = concatCases[i][1];
    trace("// " + matrix + ".concat(" + other + ")");
    trace(matrix.concat(other));
    trace(matrix);
    trace(other);
    trace("");
}

trace("// matrix.concat()");
matrix = new Matrix(1, 2, 3, 4, 5, 6);
matrix.concat();
trace(matrix);
trace("");

trace("// matrix.concat({a: 2, b: 0, c: 0, d: 3, tx: 4, ty: 5})");
matrix = new Matrix(1, 0, 0, 1, 1, 1);
matrix.concat({a: 2, b: 0, c: 0, d: 3, tx: 4, ty: 5});
trace(matrix);
trace("");

trace("// matrix.concat(other), no skew");
matrix = tracingMatrix("matrix", 2, 0, 0, 3, 5, 7);
other = tracingMatrix("other", 11, 0, 0, 13, 17, 19);
matrix.concat(other);
trace("");

trace("// matrix.concat(other), with skew");
matrix = tracingMatrix("matrix", 2, 1, 0, 3, 5, 7);
other = tracingMatrix("other", 11, 0, 1, 13, 17, 19);
matrix.concat(other);
trace("");

trace("// matrix.concat(other), with non-number zero skew");
matrix = tracingMatrix("matrix", 2, "0", false, 3, Infinity, 7);
other = tracingMatrix("other", 11, tracingNumber("other.b", 0), "0", 13, 17, 19);
matrix.concat(other);
trace(matrix);
trace("");
trace("");

trace("/// invert");

var invertCases = [
    new Matrix(2, 3, 5, 7, 9, 11),
    new Matrix(2, 0, 0, 4, 8, 16),
    new Matrix(0, 0, 0, 4, 8, 16),
    new Matrix(0, 0, 0, 0, 8, 16),
    new Matrix(1, 2, 2, 4, 8, 16),
    new Matrix(0, 1, 1, 0, 8, 16),
    new Matrix(NaN, 0, 0, 2, 3, 4),
    new Matrix(NaN, 1, 1, 2, 3, 4),
    new Matrix("2", "0", "0", "4", "8", "16"),
    new Matrix(1)
];
for (i = 0; i < invertCases.length; i++) {
    matrix = invertCases[i];
    trace("// " + matrix + ".invert()");
    trace(matrix.invert());
    trace(matrix);
    trace("");
}

trace("// matrix.invert(), no skew");
matrix = tracingMatrixWithMethods("matrix", 2, 0, 0, 4, 8, 16);
matrix.invert();
trace("");

trace("// matrix.invert(), with skew");
matrix = tracingMatrixWithMethods("matrix", 2, 3, 5, 7, 9, 11);
matrix.invert();
trace("");

trace("// matrix.invert(), singular");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 2, 4, 8, 16);
matrix.invert();
trace("");

trace("// matrix.invert(), with non-number zero skew");
matrix = tracingMatrixWithMethods("matrix", 2, "0", false, 4, 8, 16);
matrix.invert();
trace("");

trace("// matrix.invert(), with valueOf zero skew");
matrix = tracingMatrixWithMethods("matrix", 2, tracingNumber("b", 0), tracingNumber("c", 0), 4, 8, 16);
matrix.invert();
trace("");

trace("// matrix.invert() with overridden deltaTransformPoint");
matrix = new Matrix(2, 3, 5, 7, 9, 11);
matrix.deltaTransformPoint = function(pt) {
    trace("  deltaTransformPoint(" + pt + ")");
    return {x: "x", y: "y"};
};
matrix.invert();
trace(matrix);
trace("");
trace("");

trace("/// createBox");

var createBoxArgs = [
    [2, 3],
    [2, 3, 0],
    [2, 3, undefined],
    [2, 3, Math.PI / 2],
    [2, 3, 0, 4],
    [2, 3, 0, 4, 5],
    [2, 3, 0, undefined, undefined],
    [2, 3, 0, 4, 5, 6]
];
for (i = 0; i < createBoxArgs.length; i++) {
    matrix = new Matrix(1, 2, 3, 4, 5, 6);
    trace("// matrix.createBox(" + createBoxArgs[i].join(", ") + ")");
    trace(matrix.createBox.apply(matrix, createBoxArgs[i]));
    trace(matrix);
    trace("");
}

trace("// matrix.createBox(2, 3, 0, 4, 5)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.createBox(2, 3, 0, 4, 5);
trace("");

trace("// matrix.createBox(2, 3)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.createBox(2, 3);
trace("");
trace("");

trace("/// createGradientBox");

var createGradientBoxArgs = [
    [200],
    [200, 300],
    [200, 300, 0],
    [200, 300, undefined],
    [200, 300, Math.PI / 2],
    [200, 300, 0, 10],
    [200, 300, 0, 10, 20],
    [200, 300, 0, "10", "20"],
    [200, 300, 0, 10, 20, 30]
];
for (i = 0; i < createGradientBoxArgs.length; i++) {
    matrix = new Matrix(1, 2, 3, 4, 5, 6);
    trace("// matrix.createGradientBox(" + createGradientBoxArgs[i].join(", ") + ")");
    trace(matrix.createGradientBox.apply(matrix, createGradientBoxArgs[i]));
    trace(matrix);
    trace("");
}

trace("// matrix.createGradientBox(200, 300)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.createGradientBox(200, 300);
trace("");

trace("// matrix.createGradientBox(200, 300, 1, 10, 20)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.createGradientBox(200, 300, 1, 10, 20);
trace("");

trace("// matrix.createGradientBox(width, height, rotation, x, y) with valueOf");
matrix = new Matrix();
matrix.createGradientBox(tracingNumber("width", 200), tracingNumber("height", 300),
    tracingNumber("rotation", 0), tracingNumber("x", 10), tracingNumber("y", 20));
trace(matrix);
trace("");
trace("");

trace("/// rotate");

var rotateArgs = [0, Math.PI / 2, 1, undefined, "1"];
for (i = 0; i < rotateArgs.length; i++) {
    matrix = new Matrix(1, 2, 3, 4, 5, 6);
    trace("// matrix.rotate(" + rotateArgs[i] + ")");
    trace(matrix.rotate(rotateArgs[i]));
    trace(matrix);
    trace("");
}

trace("// matrix.rotate(1)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.rotate(1);
trace("");
trace("");

trace("/// scale");

var scaleArgs = [
    [3, 5],
    [2],
    [0, 0],
    ["2", "3"]
];
for (i = 0; i < scaleArgs.length; i++) {
    matrix = new Matrix(2, 0, 0, 2, 100, 100);
    trace("// " + matrix + ".scale(" + scaleArgs[i].join(", ") + ")");
    trace(matrix.scale.apply(matrix, scaleArgs[i]));
    trace(matrix);
    trace("");
    matrix = new Matrix(1, 2, 3, 4, 5, 6);
    trace("// " + matrix + ".scale(" + scaleArgs[i].join(", ") + ")");
    matrix.scale.apply(matrix, scaleArgs[i]);
    trace(matrix);
    trace("");
}

trace("// matrix.scale(2, 3)");
matrix = tracingMatrixWithMethods("matrix", 1, 2, 3, 4, 5, 6);
matrix.scale(2, 3);
trace("");
trace("");

trace("/// translate");

var translateArgs = [
    [3, 5],
    [2],
    ["2", "3"]
];
for (i = 0; i < translateArgs.length; i++) {
    matrix = new Matrix(2, 0, 0, 2, 100, 100);
    trace("// " + matrix + ".translate(" + translateArgs[i].join(", ") + ")");
    trace(matrix.translate.apply(matrix, translateArgs[i]));
    trace(matrix);
    trace("");
}

trace("// matrix.translate(1, 2)");
matrix = tracingMatrix("matrix", 1, 2, 3, 4, 5, 6);
matrix.translate(1, 2);
trace("");
trace("");

var pointCases = [undefined, new Point(1, 1), {x: 2, y: 3}, {x: "2", y: "3"}];
var pointLabels = ["undefined", "new Point(1, 1)", "{x: 2, y: 3}", "{x: '2', y: '3'}"];

trace("/// transformPoint");

matrix = new Matrix(2, 3, 5, 7, 11, 13);
for (i = 0; i < pointCases.length; i++) {
    trace("// matrix.transformPoint(" + pointLabels[i] + ")");
    result = matrix.transformPoint(pointCases[i]);
    trace(result);
    trace(result instanceof Point);
    trace("");
}

trace("// new Matrix(2, 3, 5, 7, '11', '13').transformPoint(new Point(1, 1))");
trace((new Matrix(2, 3, 5, 7, "11", "13")).transformPoint(new Point(1, 1)));
trace("");

trace("// matrix.transformPoint(other)");
matrix = tracingMatrix("matrix", 2, 3, 5, 7, 11, 13);
other = tracingPoint("other", 1, 1);
trace(matrix.transformPoint(other));
trace("");
trace("");

trace("/// deltaTransformPoint");

matrix = new Matrix(2, 3, 5, 7, 11, 13);
for (i = 0; i < pointCases.length; i++) {
    trace("// matrix.deltaTransformPoint(" + pointLabels[i] + ")");
    result = matrix.deltaTransformPoint(pointCases[i]);
    trace(result);
    trace(result instanceof Point);
    trace("");
}

trace("// matrix.deltaTransformPoint(other)");
matrix = tracingMatrix("matrix", 2, 3, 5, 7, 11, 13);
other = tracingPoint("other", 1, 1);
trace(matrix.deltaTransformPoint(other));
trace("");
trace("");

trace("/// toString");

trace("// new Matrix(1.5, -2, NaN, Infinity, 'a', null)");
trace((new Matrix(1.5, -2, NaN, Infinity, "a", null)).toString());
trace("");

trace("// Matrix.prototype.toString.call({a: 1, b: 2, c: 3, d: 4, tx: 5, ty: 6})");
trace(Matrix.prototype.toString.call({a: 1, b: 2, c: 3, d: 4, tx: 5, ty: 6}));
trace("");

trace("// matrix.toString()");
matrix = tracingMatrix("matrix", 1, 2, 3, 4, 5, 6);
trace(matrix.toString());
trace("");
