flash.geom.Point = function(x, y) {
    // Only a no-argument call defaults to (0, 0); `new Point(1)` keeps `y` undefined.
    if (arguments.length === 0) {
        // y is assigned before x, which affects property enumeration order.
        this.y = 0;
        this.x = 0;
    } else {
        // x is assigned before y, which affects property enumeration order.
        this.x = x;
        this.y = y;
    }
};

var o = flash.geom.Point;

o.distance = function(pt1, pt2) {
    // Dispatch through `pt1.subtract`, which can be overridden.
    var diff = pt1.subtract(pt2);
    return diff.length;
};

o.polar = function(len, angle) {
    // y is computed before x, which is observable if `Math.sin` or `Math.cos` are overridden.
    var y = len * Math.sin(angle);
    var x = len * Math.cos(angle);
    return new flash.geom.Point(x, y);
};

o.interpolate = function(pt1, pt2, f) {
    // y is computed before x, which is observable via property accesses.
    // The `+` operator concatenates when `pt2`'s coordinates are strings.
    var y = pt2.y + (pt1.y - pt2.y) * f;
    var x = pt2.x + (pt1.x - pt2.x) * f;
    return new flash.geom.Point(x, y);
};

var o = flash.geom.Point.prototype;

o.addProperty("length", function() {
    // Recomputed on every read; x is read before y.
    var x2 = this.x * this.x;
    var y2 = this.y * this.y;
    return Math.sqrt(x2 + y2);
}, function(_) {
    // No-op setter; the length property is read-only.
});

o.clone = function() {
    // y is read before x, which is observable via property accesses.
    var y = this.y;
    var x = this.x;
    return new flash.geom.Point(x, y);
};

o.offset = function(dx, dy) {
    // x is updated before y, which is observable via property accesses.
    this.x += dx;
    this.y += dy;
};

o.equals = function(toCompare) {
    // The `instanceof` check comes first, so plain objects never compare equal and their properties are never read.
    // Comparison order is observable via property accesses, and the `==` operator makes "1" equal 1.
    return toCompare instanceof flash.geom.Point &&
        toCompare.x == this.x &&
        toCompare.y == this.y;
};

o.subtract = function(v) {
    // y is read before x, which is observable via property accesses.
    var y = this.y - v.y;
    var x = this.x - v.x;
    return new flash.geom.Point(x, y);
};

o.add = function(v) {
    // y is read before x, which is observable via property accesses.
    var y = this.y + v.y;
    var x = this.x + v.x;
    return new flash.geom.Point(x, y);
};

o.normalize = function(length) {
    var currentLength = this.length;
    // Bail out if current length is zero, negative, or NaN.
    if (currentLength <= 0) {
        return;
    }

    // A negative `length` flips the direction; an infinite current length scales to (0, 0).
    var scale = length / currentLength;
    this.x *= scale;
    this.y *= scale;
};

o.toString = function() {
    return "(x=" + this.x + ", y=" + this.y + ")";
};
