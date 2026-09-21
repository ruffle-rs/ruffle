import flash.geom.Point;
import flash.geom.Rectangle;

function dump(rect) {
    trace("(top=" + rect.top + ", right=" + rect.right + ", bottom=" + rect.bottom + ", left=" + rect.left
        + ", topLeft=" + rect.topLeft + ", bottomRight=" + rect.bottomRight
        + ", width=" + rect.width + ", height=" + rect.height + ", size=" + rect.size
        + ", x=" + rect.x + ", y=" + rect.y + ")");
}

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

function tracingRect(name, x, y, width, height) {
    var rect = new Rectangle();
    addTracingProperty(name, rect, "x", x);
    addTracingProperty(name, rect, "y", y);
    addTracingProperty(name, rect, "width", width);
    addTracingProperty(name, rect, "height", height);
    return rect;
}

function tracingPoint(name, x, y) {
    var point = new Point();
    addTracingProperty(name, point, "x", x);
    addTracingProperty(name, point, "y", y);
    return point;
}

function tracingMethod(name, obj, method) {
    var original = obj[method];
    obj[method] = function() {
        trace("  call " + name + "." + method + "()");
        return original.apply(this, arguments);
    };
}

function tracingRectWithMethods(name, x, y, width, height) {
    var rect = tracingRect(name, x, y, width, height);
    tracingMethod(name, rect, "isEmpty");
    tracingMethod(name, rect, "setEmpty");
    tracingMethod(name, rect, "clone");
    tracingMethod(name, rect, "intersection");
    return rect;
}

var tracingNumber = {valueOf: function() {
    trace("  valueOf");
    return 10;
}};

trace("/// Constructors");

trace("// new Rectangle()");
dump(new Rectangle());
trace("");

trace("// new Rectangle(1)");
dump(new Rectangle(1));
trace("");

trace("// new Rectangle(1, 2, 3, 4)");
dump(new Rectangle(1, 2, 3, 4));
trace("");

trace("// new Rectangle('1', '2', '3', '4')");
dump(new Rectangle("1", "2", "3", "4"));
trace("");
trace("");

trace("/// Property enumeration order");

trace("// new Rectangle()");
trace(keys(new Rectangle()));
trace("");

trace("// new Rectangle(1, 2, 3, 4)");
trace(keys(new Rectangle(1, 2, 3, 4)));
trace("");

trace("// new Rectangle(1, 2, 3, 4) after setEmpty()");
var rect = new Rectangle(1, 2, 3, 4);
rect.setEmpty();
trace(keys(rect));
trace("");

trace("// new Rectangle(1, 2, 3, 4).clone()");
trace(keys((new Rectangle(1, 2, 3, 4)).clone()));
trace("");

trace("// new Rectangle(1, 2, 3, 4).intersection(new Rectangle(2, 3, 4, 5))");
trace(keys((new Rectangle(1, 2, 3, 4)).intersection(new Rectangle(2, 3, 4, 5))));
trace("");

trace("// new Rectangle(1, 2, 3, 4).union(new Rectangle(2, 3, 4, 5))");
trace(keys((new Rectangle(1, 2, 3, 4)).union(new Rectangle(2, 3, 4, 5))));
trace("");

trace("// Rectangle.prototype");
trace(keys(Rectangle.prototype));
trace("");
trace("");

trace("/// Setters");

rect = new Rectangle(1, 3, 5, 7);
trace("// before modifications");
dump(rect);
trace("");
trace("");

var values = [-200, NaN, undefined];
var i;

trace("/// top");

for (i = 0; i < values.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.top = values[i];
    trace("// rect.top = " + values[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// right");

for (i = 0; i < values.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.right = values[i];
    trace("// rect.right = " + values[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// left");

for (i = 0; i < values.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.left = values[i];
    trace("// rect.left = " + values[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// bottom");

for (i = 0; i < values.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.bottom = values[i];
    trace("// rect.bottom = " + values[i]);
    dump(rect);
    trace("");
}
trace("");

var points = [new Point(-100, -200), {x: "1", y: "2"}, undefined];
var pointLabels = ["new Point(-100, -200)", "{x: '1', y: '2'}", "undefined"];

trace("/// topLeft");

for (i = 0; i < points.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.topLeft = points[i];
    trace("// rect.topLeft = " + pointLabels[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// bottomRight");

for (i = 0; i < points.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.bottomRight = points[i];
    trace("// rect.bottomRight = " + pointLabels[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// size");

for (i = 0; i < points.length; i++) {
    rect = new Rectangle(1, 3, 5, 7);
    rect.size = points[i];
    trace("// rect.size = " + pointLabels[i]);
    dump(rect);
    trace("");
}
trace("");

trace("/// Point getters return fresh objects");

rect = new Rectangle(1, 3, 5, 7);
trace("// rect.topLeft === rect.topLeft");
trace(rect.topLeft === rect.topLeft);
trace("");

trace("// rect.topLeft = point; point.x = 100");
var point = new Point(10, 20);
rect.topLeft = point;
point.x = 100;
dump(rect);
trace("");
trace("");

trace("/// Getters: property access order");

var self = tracingRect("self", 1, 3, 5, 7);

trace("// self.right");
trace(self.right);
trace("");

trace("// self.bottomRight");
trace(self.bottomRight);
trace("");
trace("");

trace("/// Setters: property access order");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.left = 0");
self.left = 0;
trace("");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.right = 0");
self.right = 0;
trace("");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.topLeft = other");
var other = tracingPoint("other", 10, 20);
self.topLeft = other;
trace("");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.bottomRight = other");
other = tracingPoint("other", 10, 20);
self.bottomRight = other;
trace("");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.size = other");
other = tracingPoint("other", 10, 20);
self.size = other;
trace("");
trace("");

trace("/// clone");

rect = new Rectangle(1, 3, 5, 7);
trace("// rect.clone() === rect");
trace(rect.clone() === rect);
trace("");

trace("// Rectangle.prototype.clone.call({x: 1, y: 2, width: 3, height: 4})");
var clone = Rectangle.prototype.clone.call({x: 1, y: 2, width: 3, height: 4});
trace(clone);
trace(clone instanceof Rectangle);
trace("");

trace("// self.clone()");
self = tracingRect("self", 1, 3, 5, 7);
trace(self.clone());
trace("");
trace("");

trace("/// equals");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.equals(new Rectangle(1, 3, 5, 7))");
trace(rect.equals(new Rectangle(1, 3, 5, 7)));
trace("");

trace("// rect.equals(new Rectangle('1', '3', '5', '7'))");
trace(rect.equals(new Rectangle("1", "3", "5", "7")));
trace("");

trace("// rect.equals({x: 1, y: 3, width: 5, height: 7})");
trace(rect.equals({x: 1, y: 3, width: 5, height: 7}));
trace("");

trace("// rect.equals({x: 1, y: 3, width: 5, height: 7, __proto__: Rectangle.prototype})");
var temp = {x: 1, y: 3, width: 5, height: 7};
temp.__proto__ = Rectangle.prototype;
trace(rect.equals(temp));
trace("");

trace("// Rectangle.prototype.equals.call({x: 1, y: 3, width: 5, height: 7}, rect)");
trace(Rectangle.prototype.equals.call({x: 1, y: 3, width: 5, height: 7}, rect));
trace("");
trace("");

trace("/// equals: property access order");

self = tracingRect("self", 1, 3, 5, 7);

trace("// self.equals(other), all equal");
other = tracingRect("other", 1, 3, 5, 7);
trace(self.equals(other));
trace("");

trace("// self.equals(other), other is a plain object");
other = {};
addTracingProperty("other", other, "x", 1);
addTracingProperty("other", other, "y", 3);
addTracingProperty("other", other, "width", 5);
addTracingProperty("other", other, "height", 7);
trace(self.equals(other));
trace("");
trace("");

trace("/// isEmpty");

trace("// new Rectangle(1, 2, 3, 4).isEmpty()");
trace((new Rectangle(1, 2, 3, 4)).isEmpty());
trace("");

trace("// new Rectangle(1, 2, 3, 0).isEmpty()");
trace((new Rectangle(1, 2, 3, 0)).isEmpty());
trace("");

trace("// new Rectangle(1, 2, 0, 4).isEmpty()");
trace((new Rectangle(1, 2, 0, 4)).isEmpty());
trace("");

trace("// new Rectangle(1, 2, NaN, 4).isEmpty()");
trace((new Rectangle(1, 2, NaN, 4)).isEmpty());
trace("");

trace("// new Rectangle(1, 2, '3', '4').isEmpty()");
trace((new Rectangle(1, 2, "3", "4")).isEmpty());
trace("");

trace("// self.isEmpty()");
self = tracingRect("self", 1, 3, 5, 7);
trace(self.isEmpty());
trace("");
trace("");

trace("/// setEmpty");

rect = new Rectangle(1, 3, 5, 7);
trace("// rect.setEmpty()");
trace(rect.setEmpty());
dump(rect);
trace("");

trace("// self.setEmpty()");
self = tracingRect("self", 1, 3, 5, 7);
self.setEmpty();
trace("");
trace("");

trace("/// contains");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.contains()");
trace(rect.contains());
trace("");

trace("// rect.contains(1, 3)");
trace(rect.contains(1, 3));
trace("");

trace("// rect.contains(6, 10)");
trace(rect.contains(6, 10));
trace("");

trace("// rect.contains(4, NaN)");
trace(rect.contains(4, NaN));
trace("");

trace("// rect.contains('3', '5')");
trace(rect.contains("3", "5"));
trace("");
trace("");

trace("/// contains: property access order");

self = tracingRect("self", 1, 3, 5, 7);
trace("// self.contains(3, 5), inside");
trace(self.contains(3, 5));
trace("");

trace("// self.contains(0, 5), left of x");
trace(self.contains(0, 5));
trace("");

trace("// self.contains(NaN, 5)");
trace(self.contains(NaN, 5));
trace("");
trace("");

trace("/// containsPoint");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.containsPoint()");
trace(rect.containsPoint());
trace("");

trace("// rect.containsPoint(new Point(1, 3))");
trace(rect.containsPoint(new Point(1, 3)));
trace("");

trace("// rect.containsPoint(new Point(6, 10))");
trace(rect.containsPoint(new Point(6, 10)));
trace("");

trace("// rect.containsPoint({x: 2, y: 5})");
trace(rect.containsPoint({x: 2, y: 5}));
trace("");

trace("// self.containsPoint(other)");
self = tracingRect("self", 1, 3, 5, 7);
other = tracingPoint("other", 3, 5);
trace(self.containsPoint(other));
trace("");
trace("");

trace("/// containsRectangle");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.containsRectangle()");
trace(rect.containsRectangle());
trace("");

trace("// rect.containsRectangle(rect)");
trace(rect.containsRectangle(rect));
trace("");

trace("// rect.containsRectangle(new Rectangle(1, 3, 5.1, 7.1))");
trace(rect.containsRectangle(new Rectangle(1, 3, 5.1, 7.1)));
trace("");

trace("// rect.containsRectangle(new Rectangle(2, undefined, 1, 1))");
trace(rect.containsRectangle(new Rectangle(2, undefined, 1, 1)));
trace("");

trace("// self.containsRectangle(other)");
self = tracingRect("self", 1, 3, 5, 7);
other = tracingRect("other", 2, 4, 3, 5);
trace(self.containsRectangle(other));
trace("");
trace("");

trace("/// inflate");

trace("// rect.inflate()");
rect = new Rectangle(1, 3, 5, 7);
trace(rect.inflate());
dump(rect);
trace("");

trace("// rect.inflate(1, 2)");
rect = new Rectangle(1, 3, 5, 7);
rect.inflate(1, 2);
dump(rect);
trace("");

trace("// rect.inflate('1', '2')");
rect = new Rectangle(1, 3, 5, 7);
rect.inflate("1", "2");
dump(rect);
trace("");

trace("// self.inflate(tracingNumber, tracingNumber)");
self = tracingRect("self", 1, 3, 5, 7);
self.inflate(tracingNumber, tracingNumber);
trace("");
trace("");

trace("/// inflatePoint");

trace("// rect.inflatePoint()");
rect = new Rectangle(1, 3, 5, 7);
trace(rect.inflatePoint());
dump(rect);
trace("");

trace("// rect.inflatePoint(new Point(1, 2))");
rect = new Rectangle(1, 3, 5, 7);
rect.inflatePoint(new Point(1, 2));
dump(rect);
trace("");

trace("// rect.inflatePoint({x: 5})");
rect = new Rectangle(1, 3, 5, 7);
rect.inflatePoint({x: 5});
dump(rect);
trace("");

trace("// self.inflatePoint(other)");
self = tracingRect("self", 1, 3, 5, 7);
other = tracingPoint("other", 1, 2);
self.inflatePoint(other);
trace("");
trace("");

trace("/// offset");

trace("// rect.offset()");
rect = new Rectangle(1, 3, 5, 7);
trace(rect.offset());
dump(rect);
trace("");

trace("// rect.offset(1, 2)");
rect = new Rectangle(1, 3, 5, 7);
rect.offset(1, 2);
dump(rect);
trace("");

trace("// rect.offset('1', '2')");
rect = new Rectangle(1, 3, 5, 7);
rect.offset("1", "2");
dump(rect);
trace("");

trace("// self.offset(tracingNumber, tracingNumber)");
self = tracingRect("self", 1, 3, 5, 7);
self.offset(tracingNumber, tracingNumber);
trace("");
trace("");

trace("/// offsetPoint");

trace("// rect.offsetPoint()");
rect = new Rectangle(1, 3, 5, 7);
trace(rect.offsetPoint());
dump(rect);
trace("");

trace("// rect.offsetPoint(new Point(1, 2))");
rect = new Rectangle(1, 3, 5, 7);
rect.offsetPoint(new Point(1, 2));
dump(rect);
trace("");

trace("// rect.offsetPoint({x: 5})");
rect = new Rectangle(1, 3, 5, 7);
rect.offsetPoint({x: 5});
dump(rect);
trace("");

trace("// self.offsetPoint(other)");
self = tracingRect("self", 1, 3, 5, 7);
other = tracingPoint("other", 1, 2);
self.offsetPoint(other);
trace("");
trace("");

trace("/// intersection");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.intersection()");
trace(rect.intersection());
trace("");

trace("// rect.intersection(new Rectangle(3, 5, 7, 9))");
trace(rect.intersection(new Rectangle(3, 5, 7, 9)));
trace("");

trace("// rect.intersection(new Rectangle(6, 5, 1, 1))");
trace(rect.intersection(new Rectangle(6, 5, 1, 1)));
trace("");

trace("// rect.intersection(new Rectangle(2, 4, 0, 0))");
trace(rect.intersection(new Rectangle(2, 4, 0, 0)));
trace("");

trace("// new Rectangle().intersection(rect)");
trace((new Rectangle()).intersection(rect));
trace("");

trace("// rect.intersection(new Rectangle(NaN, 5, 1, 1))");
trace(rect.intersection(new Rectangle(NaN, 5, 1, 1)));
trace("");

trace("// rect.intersection(new Rectangle('3', '5', '7', '9'))");
trace(rect.intersection(new Rectangle("3", "5", "7", "9")));
trace("");

trace("// rect.intersection({x: 3, y: 5, width: 7, height: 1})");
trace(rect.intersection({x: 3, y: 5, width: 7, height: 1}));
trace("");
trace("");

trace("/// intersection: property access order");

self = tracingRectWithMethods("self", 1, 3, 5, 7);

trace("// self.intersection(other), overlapping");
other = tracingRectWithMethods("other", 3, 5, 7, 9);
trace(self.intersection(other));
trace("");

trace("// self.intersection(other), other is empty");
other = tracingRectWithMethods("other", 3, 5, 0, 9);
trace(self.intersection(other));
trace("");
trace("");

trace("/// intersects");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.intersects(new Rectangle(3, 5, 7, 9))");
trace(rect.intersects(new Rectangle(3, 5, 7, 9)));
trace("");

trace("// rect.intersects(new Rectangle(6, 5, 1, 1))");
trace(rect.intersects(new Rectangle(6, 5, 1, 1)));
trace("");

trace("// self.intersects(other)");
self = tracingRectWithMethods("self", 1, 3, 5, 7);
other = tracingRectWithMethods("other", 3, 5, 7, 9);
trace(self.intersects(other));
trace("");
trace("");

trace("/// union");

rect = new Rectangle(1, 3, 5, 7);

trace("// rect.union()");
trace(rect.union());
trace("");

trace("// rect.union(new Rectangle(3, 5, 7, 9))");
trace(rect.union(new Rectangle(3, 5, 7, 9)));
trace("");

trace("// rect.union(new Rectangle(2, 4, 0, 0))");
trace(rect.union(new Rectangle(2, 4, 0, 0)));
trace("");

trace("// new Rectangle().union(rect) === rect");
trace((new Rectangle()).union(rect) === rect);
trace("");

trace("// rect.union(new Rectangle('3', '5', '7', '9'))");
trace(rect.union(new Rectangle("3", "5", "7", "9")));
trace("");

trace("// rect.union({x: 3, y: 5, width: 7, height: 1})");
trace(rect.union({x: 3, y: 5, width: 7, height: 1}));
trace("");

trace("// new Rectangle().union({x: 3, y: 5, width: 7, height: 1})");
trace((new Rectangle()).union({x: 3, y: 5, width: 7, height: 1}));
trace("");
trace("");

trace("/// union: property access order");

self = tracingRectWithMethods("self", 1, 3, 5, 7);

trace("// self.union(other), other is empty");
other = tracingRectWithMethods("other", 3, 5, 0, 9);
trace(self.union(other));
trace("");

trace("// self.union(other), self is empty");
self = tracingRectWithMethods("self", 1, 3, 0, 7);
other = tracingRectWithMethods("other", 3, 5, 7, 9);
trace(self.union(other));
trace("");
