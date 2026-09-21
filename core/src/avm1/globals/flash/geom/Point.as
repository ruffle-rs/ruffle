flash.geom.Point = function(x, y) {
    // The order of assignments is observable via property enumeration order.
    if (arguments.length === 0) {
        this.y = 0;
        this.x = 0;
    } else {
        this.x = x;
        this.y = y;
    }
};

var o = flash.geom.Point;

o.distance = function(pt1, pt2) {
    return pt1.subtract(pt2).length;
};

o.polar = function(len, angle) {
    return new flash.geom.Point(len * Math.cos(angle), len * Math.sin(angle));
};

o.interpolate = function(pt1, pt2, f) {
    return new flash.geom.Point(pt2.x + f * (pt1.x - pt2.x), pt2.y + f * (pt1.y - pt2.y));
};

var o = flash.geom.Point.prototype;

o.addProperty("length", function() {
    return Math.sqrt(this.x * this.x + this.y * this.y);
}, function(_) {
    // No-op setter; the length property is read-only.
});

o.clone = function() {
    return new flash.geom.Point(this.x, this.y);
};

o.offset = function(dx, dy) {
    this.x += dx;
    this.y += dy;
};

o.equals = function(toCompare) {
    return toCompare instanceof flash.geom.Point && toCompare.x == this.x && toCompare.y == this.y;
};

o.subtract = function(v) {
    return new flash.geom.Point(this.x - v.x, this.y - v.y);
};

o.add = function(v) {
    return new flash.geom.Point(this.x + v.x, this.y + v.y);
};

o.normalize = function(length) {
    var currentLength = this.length;
    if (currentLength > 0) {
        var scale = length / currentLength;
        this.x *= scale;
        this.y *= scale;
    }
};

o.toString = function() {
    return "(x=" + this.x + ", y=" + this.y + ")";
};
