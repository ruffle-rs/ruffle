flash.geom.Rectangle = function(x, y, width, height) {
    // Only a no-argument call defaults to an empty rectangle; `new Rectangle(1)` keeps the rest undefined.
    if (arguments.length === 0) {
        this.setEmpty();
        return;
    }

    // x, y, width, and height are assigned in order, which affects property enumeration order.
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
};

var o = flash.geom.Rectangle.prototype;

o.clone = function() {
    // x, y, width, and height are read in reverse order, which is observable via property accesses.
    var height = this.height;
    var width = this.width;
    var y = this.y;
    var x = this.x;
    return new flash.geom.Rectangle(x, y, width, height);
};

o.setEmpty = function() {
    // x, y, width, and height are assigned in reverse order, which affects property enumeration order.
    this.height = 0;
    this.width = 0;
    this.y = 0;
    this.x = 0;
};

o.isEmpty = function() {
    // width is read before height, and height is not read at all if width is non-positive.
    // The `<=` operator is true for NaN, so a NaN width or height makes the rectangle empty.
    return this.width <= 0 || this.height <= 0;
};

o.addProperty("left", function() {
    return this.x;
}, function(left) {
    // width is updated before x, which is observable via property accesses.
    this.width += this.x - left;
    this.x = left;
});

o.addProperty("right", function() {
    // The `+` operator concatenates when the coordinates are strings.
    return this.x + this.width;
}, function(right) {
    this.width = right - this.x;
});

o.addProperty("top", function() {
    return this.y;
}, function(top) {
    // height is updated before y, which is observable via property accesses.
    this.height += this.y - top;
    this.y = top;
});

o.addProperty("bottom", function() {
    // The `+` operator concatenates when the coordinates are strings.
    return this.y + this.height;
}, function(bottom) {
    this.height = bottom - this.y;
});

o.addProperty("topLeft", function() {
    // y is read before x, which is observable via property accesses.
    var y = this.y;
    var x = this.x;
    return new flash.geom.Point(x, y);
}, function(topLeft) {
    // width and height are updated before x and y, which is observable via property accesses.
    this.width += this.x - topLeft.x;
    this.height += this.y - topLeft.y;
    this.x = topLeft.x;
    this.y = topLeft.y;
});

o.addProperty("bottomRight", function() {
    // The bottom is computed before the right, which is observable via property accesses.
    var bottom = this.y + this.height;
    var right = this.x + this.width;
    return new flash.geom.Point(right, bottom);
}, function(bottomRight) {
    // width is updated before height, which is observable via property accesses.
    this.width = bottomRight.x - this.x;
    this.height = bottomRight.y - this.y;
});

o.addProperty("size", function() {
    // height is read before width, which is observable via property accesses.
    var height = this.height;
    var width = this.width;
    return new flash.geom.Point(width, height);
}, function(size) {
    // width is updated before height, which is observable via property accesses.
    this.width = size.x;
    this.height = size.y;
});

o.inflate = function(dx, dy) {
    // x and width are updated before y and height, which is observable via property accesses.
    this.x -= dx;
    this.width += dx * 2;
    this.y -= dy;
    this.height += dy * 2;
};

o.inflatePoint = function(pt) {
    // x and width are updated before y and height, which is observable via property accesses.
    this.x -= pt.x;
    this.width += pt.x * 2;
    this.y -= pt.y;
    this.height += pt.y * 2;
};

o.offset = function(dx, dy) {
    // x is updated before y, which is observable via property accesses.
    this.x += dx;
    this.y += dy;
};

o.offsetPoint = function(pt) {
    // x is updated before y, which is observable via property accesses.
    this.x += pt.x;
    this.y += pt.y;
};

o.contains = function(x, y) {
    // Comparison order is observable via property accesses.
    // A comparison with NaN yields `undefined`, which the `&&` operator passes through as the result.
    return x >= this.x &&
        x < this.x + this.width &&
        y >= this.y &&
        y < this.y + this.height;
};

o.containsPoint = function(pt) {
    // Comparison order is observable via property accesses.
    return pt.x >= this.x &&
        pt.x < this.x + this.width &&
        pt.y >= this.y &&
        pt.y < this.y + this.height;
};

o.containsRectangle = function(rect) {
    // The edges of `rect` are computed before our own, which is observable via property accesses.
    var rectRight = rect.x + rect.width;
    var rectBottom = rect.y + rect.height;
    var right = this.x + this.width;
    var bottom = this.y + this.height;
    return rect.x >= this.x &&
        rect.x < right &&
        rect.y >= this.y &&
        rect.y < bottom &&
        rectRight > this.x &&
        rectRight <= right &&
        rectBottom > this.y &&
        rectBottom <= bottom;
};

o.intersection = function(toIntersect) {
    // The four-argument constructor cannot be used here because it produces the wrong property enumeration order.
    var result = new flash.geom.Rectangle();
    // Dispatch through `isEmpty`, which can be overridden.
    if (this.isEmpty() || toIntersect.isEmpty()) {
        // Flash empties the (already empty) result once more here.
        result.setEmpty();
        return result;
    }

    // Arguments are evaluated last to first, so `toIntersect` is read before `this`.
    result.x = Math.max(this.x, toIntersect.x);
    result.y = Math.max(this.y, toIntersect.y);
    var toIntersectRight = toIntersect.x + toIntersect.width;
    var right = this.x + this.width;
    result.width = Math.min(right, toIntersectRight) - result.x;
    var toIntersectBottom = toIntersect.y + toIntersect.height;
    var bottom = this.y + this.height;
    result.height = Math.min(bottom, toIntersectBottom) - result.y;

    // Empty the result if the rectangles don't overlap.
    if (result.width <= 0 || result.height <= 0) {
        result.setEmpty();
    }

    return result;
};

o.intersects = function(toIntersect) {
    // Dispatch through `intersection`, which can be overridden.
    var result = this.intersection(toIntersect);
    return !result.isEmpty();
};

o.union = function(toUnion) {
    // Dispatch through `isEmpty` and `clone`, which can be overridden.
    if (this.isEmpty()) {
        return toUnion.clone();
    }

    if (toUnion.isEmpty()) {
        return this.clone();
    }

    // The four-argument constructor cannot be used here because it produces the wrong property enumeration order.
    var result = new flash.geom.Rectangle();
    // Arguments are evaluated last to first, so `toUnion` is read before `this`.
    result.x = Math.min(this.x, toUnion.x);
    result.y = Math.min(this.y, toUnion.y);
    var toUnionRight = toUnion.x + toUnion.width;
    var right = this.x + this.width;
    result.width = Math.max(right, toUnionRight) - result.x;
    var toUnionBottom = toUnion.y + toUnion.height;
    var bottom = this.y + this.height;
    result.height = Math.max(bottom, toUnionBottom) - result.y;
    return result;
};

o.equals = function(toCompare) {
    // The `instanceof` check comes first, so plain objects never compare equal and their properties are never read.
    // Comparison order is observable via property accesses, and the `==` operator makes "1" equal 1.
    return toCompare instanceof flash.geom.Rectangle &&
        toCompare.x == this.x &&
        toCompare.y == this.y &&
        toCompare.width == this.width &&
        toCompare.height == this.height;
};

o.toString = function() {
    return "(x=" + this.x + ", y=" + this.y + ", w=" + this.width + ", h=" + this.height + ")";
};
