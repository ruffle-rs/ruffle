import flash.geom.Point;

function addTracingProperty(name, obj, prop, value) {
    obj.addProperty(prop, function() {
        trace("  get " + name + "." + prop);
        return value;
    }, function(newValue) {
        trace("  set " + name + "." + prop + " = " + newValue);
        value = newValue;
    });
}

trace("/// Constructors");

trace("// new Point()");
trace(new Point());
trace("");

trace("// new Point(1)");
trace(new Point(1));
trace("");

trace("// new Point(1, 2)");
trace(new Point(1, 2));
trace("");

trace("// new Point({}, 2)");
var temp = {};
trace(new Point(temp, 2));
trace("");
trace("");

trace("/// Add");

point = new Point();
trace("// point.add(new Point(1, 2))");
trace(point.add(new Point(1, 2)));
trace("");

trace("// point.add({x: 3, y: 4})");
trace(point.add({x: 3, y: 4}));
trace("");

trace("// point.add(undefined)");
trace(point.add(undefined));
trace("");

trace("// point");
trace(point);
trace("");
trace("");

trace("/// Subtract");

point = new Point();
trace("// point.subtract(new Point(1, 2))");
trace(point.subtract(new Point(1, 2)));
trace("");

trace("// point.subtract({x: 3, y: 4})");
trace(point.subtract({x: 3, y: 4}));
trace("");

trace("// point.subtract(undefined)");
trace(point.subtract(undefined));
trace("");

trace("// point");
trace(point);
trace("");
trace("");

trace("/// Distance");

trace("// Point.distance(new Point(), new Point())");
trace(Point.distance(new Point(), new Point()));
trace("");

trace("// Point.distance(new Point())");
trace(Point.distance(new Point()));
trace("");

trace("// Point.distance(new Point(), new Point(), new Point())");
trace(Point.distance(new Point(), new Point(), new Point()));
trace("");

trace("// Point.distance(new Point(-100, 200), new Point(100, 200))");
trace(Point.distance(new Point(-100, 200), new Point(100, 200)));
trace("");

trace("// Point.distance({x: -100, y: 200}, new Point(100, 200))");
temp = {x: -100, y: 200};
trace(Point.distance(temp, new Point(100, 200)));
trace("");
trace("");

trace("/// Equals");

point = new Point();
trace("// point.equals(new Point(1, 2))");
trace(point.equals(new Point(1, 2)));
trace("");

trace("// point.equals(point)");
trace(point.equals(point));
trace("");

trace("// point.equals({x: 3, y: 4})");
trace(point.equals({x: 3, y: 4}));
trace("");

trace("// point.equals({x: 0, y: 0})");
trace(point.equals({x: 0, y: 0}));
trace("");

trace("// point.equals({x: 0, y: 0, __proto__: Point.prototype})");
temp = {x: 0, y: 0};
temp.__proto__ = Point.prototype;
trace(point.equals(temp));
trace("");

trace("// point.equals(null)");
trace(point.equals(null));
trace("");

trace("// new Point('1', '2').equals(new Point(1, 2))");
trace((new Point("1", "2")).equals(new Point(1, 2)));
trace("");

trace("// new Point(NaN, NaN).equals(new Point(NaN, NaN))");
trace((new Point(NaN, NaN)).equals(new Point(NaN, NaN)));
trace("");

trace("// Point.prototype.equals.call({x: 0, y: 0}, new Point())");
trace(Point.prototype.equals.call({x: 0, y: 0}, new Point()));
trace("");

trace("// point");
trace(point);
trace("");
trace("");

trace("/// Equals: property access order");

var self = new Point();
addTracingProperty("self", self, "x", 1);
addTracingProperty("self", self, "y", 2);

trace("// self.equals(other), all equal");
var other = new Point();
addTracingProperty("other", other, "x", 1);
addTracingProperty("other", other, "y", 2);
trace(self.equals(other));
trace("");

trace("// self.equals(other), x differs");
other = new Point();
addTracingProperty("other", other, "x", 5);
addTracingProperty("other", other, "y", 2);
trace(self.equals(other));
trace("");
trace("");

trace("/// Clone");

point = new Point(1, 2);
clone = point.clone();
trace("// point");
trace(point);
trace("");

trace("// clone");
trace(clone);
trace("");

trace("// point === clone");
trace(point === clone);
trace("");

trace("// point.equals(clone)");
trace(point.equals(clone));
trace("");
trace("");

trace("/// Interpolate");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200), -1)");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200), -1));
trace("");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200), 0)");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200), 0));
trace("");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200), 0.5)");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200), 0.5));
trace("");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200), 1)");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200), 1));
trace("");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200), 2)");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200), 2));
trace("");

trace("// Point.interpolate(new Point(-100, -200), new Point(100, 200))");
trace(Point.interpolate(new Point(-100, -200), new Point(100, 200)));
trace("");

trace("// Point.interpolate()");
trace(Point.interpolate());
trace("");
trace("");

trace("/// Length");

trace("// new Point().length");
trace((new Point()).length);
trace("");

trace("// new Point(100, 0).length");
trace((new Point(100, 0)).length);
trace("");

trace("// new Point(0, -200).length");
trace((new Point(0, -200)).length);
trace("");
trace("");

trace("/// Normalize");

trace("// new Point().normalize(10)");
point = new Point();
point.normalize(10);
trace(point);
trace("");

trace("// new Point().normalize(-5)");
point = new Point();
point.normalize(-5);
trace(point);
trace("");

trace("// new Point(100, 200).normalize(10)");
point = new Point(100, 200);
point.normalize(10);
trace(point);
trace("");

trace("// new Point(100, 200).normalize(-5)");
point = new Point(100, 200);
point.normalize(-5);
trace(point);
trace("");

trace("// new Point(-200, 100).normalize(10)");
point = new Point(-200, 100);
point.normalize(10);
trace(point);
trace("");

trace("// new Point(-200, 100).normalize(-5)");
point = new Point(-200, 100);
point.normalize(-5);
trace(point);
trace("");

trace("// new Point(-200, 100).normalize()");
point = new Point(-200, 100);
point.normalize();
trace(point);
trace("");

trace("// new Point(NaN, 100).normalize()");
point = new Point(NaN, 100);
point.normalize();
trace(point);
trace("");

trace("// new Point(undefined, 100).normalize(3)");
point = new Point(undefined, 100);
point.normalize(3);
trace(point);
trace("");

trace("// new Point(100, null).normalize(1)");
point = new Point(100, null);
point.normalize(1);
trace(point);
trace("");

trace("// new Point(1e308, 1e308).normalize(1)");
point = new Point(1e308, 1e308);
point.normalize(1);
trace(point);
trace("");

trace("// new Point(1, 3).normalize(3.3)");
point = new Point(1, 3);
point.normalize(3.3);
trace(point);
trace("");

trace("// Point.prototype.normalize.call({x: 3, y: 4}, 10)");
temp = {x: 3, y: 4};
Point.prototype.normalize.call(temp, 10);
trace("(x=" + temp.x + ", y=" + temp.y + ")");
trace("");
trace("");

trace("/// Normalize: side effects");

var tracingLength = {valueOf: function() {
    trace("  valueOf");
    return 10;
}};

trace("// new Point(3, 4).normalize(tracingLength)");
point = new Point(3, 4);
point.normalize(tracingLength);
trace(point);
trace("");

trace("// new Point().normalize(tracingLength)");
point = new Point();
point.normalize(tracingLength);
trace(point);
trace("");

trace("// new Point(Infinity, 1).normalize(tracingLength)");
point = new Point(Infinity, 1);
point.normalize(tracingLength);
trace(point);
trace("");

trace("// self.normalize(10), self has tracing x and y");
self = new Point();
addTracingProperty("self", self, "x", 3);
addTracingProperty("self", self, "y", 4);
self.normalize(10);
trace(self);
trace("");

trace("// self.normalize(10), self has tracing x, y and length");
self = new Point();
addTracingProperty("self", self, "x", 3);
addTracingProperty("self", self, "y", 4);
addTracingProperty("self", self, "length", 5);
self.normalize(10);
trace(self);
trace("");
trace("");

trace("/// Offset");

point = new Point();
trace("// point = new Point()");
trace(point);
trace("");

point.offset(100, 200);
trace("// point.offset(100, 200)");
trace(point);
trace("");

point.offset(-1000, -2000);
trace("// point.offset(-1000, -2000)");
trace(point);
trace("");

point.offset(500);
trace("// point.offset(500)");
trace(point);
trace("");
trace("");

trace("/// Polar");

trace("// Point.polar(5, Math.atan(3/4))");
trace(Point.polar(5, Math.atan(3/4)));
trace("");

trace("// Point.polar(5)");
trace(Point.polar(5));
trace("");

trace("// Point.polar(0, Math.atan(3/4))");
trace(Point.polar(0, Math.atan(3/4)));
trace("");
