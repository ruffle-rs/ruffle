flash.geom.Rectangle = function(x, y, width, height) {
    if (arguments.length === 0) {
        this.setEmpty();
    } else {
        // The order of assignments is observable via property enumeration order.
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }
};

var o = flash.geom.Rectangle.prototype;

o.clone = function() {
    return new flash.geom.Rectangle(this.x, this.y, this.width, this.height);
};

o.setEmpty = function() {
    // The order of assignments is observable via property enumeration order.
    this.height = 0;
    this.width = 0;
    this.y = 0;
    this.x = 0;
};

o.isEmpty = function() {
    return this.width <= 0 || this.height <= 0;
};

o.addProperty("left", function() {
    return this.x;
}, function(left) {
    this.width += this.x - left;
    this.x = left;
});

o.addProperty("right", function() {
    return this.x + this.width;
}, function(right) {
    this.width = right - this.x;
});

o.addProperty("top", function() {
    return this.y;
}, function(top) {
    this.height += this.y - top;
    this.y = top;
});

o.addProperty("bottom", function() {
    return this.y + this.height;
}, function(bottom) {
    this.height = bottom - this.y;
});

o.addProperty("topLeft", function() {
    return new flash.geom.Point(this.x, this.y);
}, function(topLeft) {
    this.width += this.x - topLeft.x;
    this.height += this.y - topLeft.y;
    this.x = topLeft.x;
    this.y = topLeft.y;
});

o.addProperty("bottomRight", function() {
    return new flash.geom.Point(this.x + this.width, this.y + this.height);
}, function(bottomRight) {
    this.width = bottomRight.x - this.x;
    this.height = bottomRight.y - this.y;
});

o.addProperty("size", function() {
    return new flash.geom.Point(this.width, this.height);
}, function(size) {
    this.width = size.x;
    this.height = size.y;
});

o.inflate = function(dx, dy) {
    this.x -= dx;
    this.width += dx * 2;
    this.y -= dy;
    this.height += dy * 2;
};

o.inflatePoint = function(pt) {
    this.x -= pt.x;
    this.width += pt.x * 2;
    this.y -= pt.y;
    this.height += pt.y * 2;
};

o.offset = function(dx, dy) {
    this.x += dx;
    this.y += dy;
};

o.offsetPoint = function(pt) {
    this.x += pt.x;
    this.y += pt.y;
};

o.contains = function(x, y) {
    return x >= this.x && x < this.x + this.width
        && y >= this.y && y < this.y + this.height;
};

o.containsPoint = function(pt) {
    return pt.x >= this.x && pt.x < this.x + this.width
        && pt.y >= this.y && pt.y < this.y + this.height;
};

o.containsRectangle = function(rect) {
    var rectRight = rect.x + rect.width;
    var rectBottom = rect.y + rect.height;
    var right = this.x + this.width;
    var bottom = this.y + this.height;
    return rect.x >= this.x && rect.x < right
        && rect.y >= this.y && rect.y < bottom
        && rectRight > this.x && rectRight <= right
        && rectBottom > this.y && rectBottom <= bottom;
};

o.intersection = function(toIntersect) {
    // The four-argument constructor cannot be used here because it produces the wrong property enumeration order.
    var result = new flash.geom.Rectangle();
    if (this.isEmpty() || toIntersect.isEmpty()) {
        // Flash empties the (already empty) result once more here.
        result.setEmpty();
        return result;
    }

    result.x = Math.max(this.x, toIntersect.x);
    result.y = Math.max(this.y, toIntersect.y);
    var toIntersectRight = toIntersect.x + toIntersect.width;
    var right = this.x + this.width;
    result.width = Math.min(right, toIntersectRight) - result.x;
    var toIntersectBottom = toIntersect.y + toIntersect.height;
    var bottom = this.y + this.height;
    result.height = Math.min(bottom, toIntersectBottom) - result.y;
    if (result.width <= 0 || result.height <= 0) {
        result.setEmpty();
        return result;
    }

    return result;
};

o.intersects = function(toIntersect) {
    return !this.intersection(toIntersect).isEmpty();
};

o.union = function(toUnion) {
    if (this.isEmpty()) {
        return toUnion.clone();
    }

    if (toUnion.isEmpty()) {
        return this.clone();
    }

    // The four-argument constructor cannot be used here because it produces the wrong property enumeration order.
    var result = new flash.geom.Rectangle();
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
    return toCompare instanceof flash.geom.Rectangle
        && toCompare.x == this.x && toCompare.y == this.y
        && toCompare.width == this.width && toCompare.height == this.height;
};

o.toString = function() {
    return "(x=" + this.x + ", y=" + this.y + ", w=" + this.width + ", h=" + this.height + ")";
};
