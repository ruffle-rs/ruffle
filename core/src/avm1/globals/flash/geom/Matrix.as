flash.geom.Matrix = function(a, b, c, d, tx, ty) {
    // Only a no-argument call defaults to the identity matrix; `new Matrix(1)` keeps the rest undefined.
    if (arguments.length === 0) {
        this.identity();
        return;
    }

    // a, b, c, d, tx, and ty are assigned in order, which affects property enumeration order.
    this.a = a;
    this.b = b;
    this.c = c;
    this.d = d;
    this.tx = tx;
    this.ty = ty;
};

var o = flash.geom.Matrix.prototype;

o.concat = function(m) {
    var a = this.a * m.a;
    var d = this.d * m.d;
    var b = 0;
    var c = 0;
    var tx = this.tx * m.a + m.tx;
    var ty = this.ty * m.d + m.ty;

    // The skew terms are only computed if either matrix is skewed, which is observable via property accesses.
    // The `!=` operator is true for NaN, so a NaN skew still takes this path.
    if (this.b != 0 || this.c != 0 || m.b != 0 || m.c != 0) {
        a += this.b * m.c;
        d += this.c * m.b;
        b += this.a * m.b + this.b * m.d;
        c += this.c * m.a + this.d * m.c;
        tx += this.ty * m.c;
        ty += this.tx * m.b;
    }

    this.a = a;
    this.b = b;
    this.c = c;
    this.d = d;
    this.tx = tx;
    this.ty = ty;
};

o.invert = function() {
    if (this.b == 0 && this.c == 0) {
        // A zero scale yields an infinite one.
        this.a = 1 / this.a;
        this.d = 1 / this.d;
        this.c = 0;
        this.b = 0;
        // The translation is computed from the already inverted a and d.
        this.tx = -this.a * this.tx;
        this.ty = -this.d * this.ty;
        return;
    }

    var a = this.a;
    var b = this.b;
    var c = this.c;
    var d = this.d;
    var det = a * d - b * c;

    // A non-invertible matrix is reset to the identity matrix.
    if (det === 0) {
        this.identity();
        return;
    }

    det = 1 / det;
    this.a = d * det;
    this.b = -b * det;
    this.c = -c * det;
    this.d = a * det;
    // Dispatch through `this.deltaTransformPoint`, which can be overridden.
    // ty is read before tx, which is observable via property accesses.
    var ty = this.ty;
    var tx = this.tx;
    var pt = this.deltaTransformPoint(new flash.geom.Point(tx, ty));
    this.tx = -pt.x;
    this.ty = -pt.y;
};

o.createBox = function(scaleX, scaleY, rotation, tx, ty) {
    var argc = arguments.length;
    if (argc < 4) {
        tx = 0;
    }
    if (argc < 5) {
        ty = 0;
    }

    // Dispatch through `this.identity`, `this.rotate` and `this.scale`, which can be overridden.
    this.identity();
    // Unlike `tx` and `ty`, `rotation` is not defaulted to 0,
    // so omitting it rotates by `undefined` and fills the matrix with NaN.
    this.rotate(rotation);
    this.scale(scaleX, scaleY);
    this.tx = tx;
    this.ty = ty;
};

o.createGradientBox = function(width, height, rotation, tx, ty) {
    var argc = arguments.length;
    if (argc < 3) {
        rotation = 0;
    }
    if (argc < 4) {
        tx = 0;
    }
    if (argc < 5) {
        ty = 0;
    }

    // Dispatch through `this.createBox`, which can be overridden.
    // The arguments are computed in reverse order, which is observable via `valueOf`.
    // The `+` operator concatenates when `tx` or `ty` are strings.
    var boxY = ty + height / 2;
    var boxX = tx + width / 2;
    var scaleY = height / 1638.4;
    var scaleX = width / 1638.4;
    this.createBox(scaleX, scaleY, rotation, boxX, boxY);
};

o.clone = function() {
    // a, b, c, d, tx, and ty are read in reverse order, which is observable via property accesses.
    var ty = this.ty;
    var tx = this.tx;
    var d = this.d;
    var c = this.c;
    var b = this.b;
    var a = this.a;
    return new flash.geom.Matrix(a, b, c, d, tx, ty);
};

o.identity = function() {
    // The diagonal is assigned before the skew and the translation, each in reverse order,
    // which affects property enumeration order.
    this.d = 1;
    this.a = 1;
    this.c = 0;
    this.b = 0;
    this.ty = 0;
    this.tx = 0;
};

o.rotate = function(angle) {
    // cos is computed before sin, which is observable if `Math.cos` or `Math.sin` are overridden.
    var cos = Math.cos(angle);
    var sin = Math.sin(angle);
    // Dispatch through `this.concat`, which can be overridden.
    this.concat(new flash.geom.Matrix(cos, sin, -sin, cos, 0, 0));
};

o.translate = function(tx, ty) {
    // tx is updated before ty, which is observable via property accesses.
    this.tx += tx;
    this.ty += ty;
};

o.scale = function(sx, sy) {
    // Dispatch through `this.concat`, which can be overridden.
    this.concat(new flash.geom.Matrix(sx, 0, 0, sy, 0, 0));
};

o.deltaTransformPoint = function(pt) {
    // y is computed before x, which is observable via property accesses.
    var y = this.d * pt.y + this.b * pt.x;
    var x = this.a * pt.x + this.c * pt.y;
    return new flash.geom.Point(x, y);
};

o.transformPoint = function(pt) {
    // y is computed before x, which is observable via property accesses.
    // The `+` operator concatenates when the translation is a string.
    var y = this.d * pt.y + this.b * pt.x + this.ty;
    var x = this.a * pt.x + this.c * pt.y + this.tx;
    return new flash.geom.Point(x, y);
};

o.toString = function() {
    return "(a=" + this.a + ", b=" + this.b + ", c=" + this.c + ", d=" + this.d + ", tx=" + this.tx + ", ty=" + this.ty + ")";
};
