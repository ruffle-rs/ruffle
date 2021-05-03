package {
    public class Test {
    }
}

import flash.geom.Point;

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

var point2 = new Point();
trace("// point.add(new Point(1, 2))");
trace(point2.add(new Point(1, 2)));
trace("");

trace("// point");
trace(point2);
trace("");
trace("");

trace("/// Subtract");

var point3 = new Point();
trace("// point.subtract(new Point(1, 2))");
trace(point3.subtract(new Point(1, 2)));
trace("");

trace("// point");
trace(point3);
trace("");
trace("");

trace("/// Distance");

trace("// Point.distance(new Point(), new Point())");
trace(Point.distance(new Point(), new Point()));
trace("");

trace("// Point.distance(new Point(-100, 200), new Point(100, 200))");
trace(Point.distance(new Point(-100, 200), new Point(100, 200)));
trace("");

trace("/// Equals");

var point4 = new Point();
trace("// point.equals(new Point(1, 2))");
trace(point4.equals(new Point(1, 2)));
trace("");


trace("// point.equals(point)");
trace(point4.equals(point4));
trace("");

trace("// point");
trace(point4);
trace("");
trace("");


trace("/// Clone");

var point5 = new Point(1, 2);
var clone = point5.clone();
trace("// point");
trace(point5);
trace("");

trace("// clone");
trace(clone);
trace("");

trace("// point === clone");
trace(point5 === clone);
trace("");

trace("// point.equals(clone)");
trace(point5.equals(clone));
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

trace("/// length");
trace("new Point().length");
trace(new Point().length);
trace("");

trace("new Point(100, 0).length");
trace(new Point(100, 0).length);
trace("");

trace("new Point(0, -200).length");
trace(new Point(0, -200).length);
trace("");
trace("");

trace("/// Normalize");
trace("// new Point() normalize(10)");
var point6 = new Point();
point6.normalize(10);
trace(point6);
trace("");

trace("// new Point() normalize(-5)");
var point7 = new Point();
point7.normalize(-5);
trace(point7);
trace("");

trace("// new Point(100, 200) normalize(10)");
var point8 = new Point(100, 200);
point8.normalize(10);
trace(point8);
trace("");

trace("// new Point(100, 200) normalize(-5)");
var point9 = new Point(100, 200);
point9.normalize(-5);
trace(point9);
trace("");

trace("// new Point(-200, 100) normalize(10)");
var point10 = new Point(-200, 100);
point10.normalize(10);
trace(point10);
trace("");

trace("// new Point(-200, 100) normalize(-5)");
var point11 = new Point(-200, 100);
point11.normalize(-5);
trace(point11);
trace("");

trace("// new Point(undefined, 100) normalize(1)");
var point14 = new Point(undefined, 100);
point14.normalize(1);
trace(point14);
trace("");
trace("");

trace("// new Point(100, null) normalize(1)");
var point15 = new Point(100, null);
point15.normalize(1);
trace(point15);
trace("");
trace("");

trace("/// Offset");
var point16 = new Point();
trace("// point = new Point()");
trace(point16);
trace("");

point16.offset(100, 200);
trace("// point.offset(100, 200)");
trace(point16);
trace("");

point16.offset(-1000, -2000);
trace("// point.offset(-1000, -2000)");
trace(point16);
trace("");


trace("/// polar");
trace("// Point.polar(5, Math.atan(3/4))");
trace(Point.polar(5, Math.atan(3/4)));
trace("");

trace("// Point.polar(0, Math.atan(3/4))");
trace(Point.polar(0, Math.atan(3/4)));
trace("");
