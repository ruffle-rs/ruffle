package {
    public class Test {
    }
}

import flash.geom.Vector3D;


// because toString() does not include w, but we are interested in it
function trv(v:Vector3D) {
    trace(v + " w=" + v.w);
}


trace("/// Constructors");
trace("// new Vector3D()");
trv(new Vector3D());
trace("");

trace("// new Vector3D(1)");
trv(new Vector3D(1));
trace("");

trace("// new Vector3D(1, 2)");
trv(new Vector3D(1, 2));
trace("");

trace("// new Vector3D(1, 2, 3)");
trv(new Vector3D(1, 2, 3));
trace("");

trace("// new Vector3D(1, 2, 3, 4)");
trv(new Vector3D(1, 2, 3, 4));
trace("");

trace("// new Vector3D({}, 2)");
var temp = {};
trv(new Vector3D(temp, 2));
trace("");
trace("");


trace("/// toString");

trace("// new Vector3D().toString()");
trace(new Vector3D().toString());
trace("");

trace("// new Vector3D(1, 2, 3).toString()");
trace(new Vector3D(1, 2, 3).toString());
trace("");

trace("// new Vector3D(1, 2, 3, 4).toString()");
trace(new Vector3D(1, 2, 3, 4).toString());
trace("");
trace("");


trace("/// Constants");

trace("// Vector3D.X_AXIS");
trv(Vector3D.X_AXIS);
trace("// Vector3D.Y_AXIS");
trv(Vector3D.Y_AXIS);
trace("// Vector3D.Z_AXIS");
trv(Vector3D.Z_AXIS);
trace("")
trace("")


trace("/// copyFrom")
trace("// new Vector3D(1, 2, 3, 4).copyFrom(new Vector3D())");
var vector_cf = new Vector3D(1, 2, 3, 4);
vector_cf.copyFrom(new Vector3D());
trv(vector_cf);
trace("")

trace("// new Vector3D().copyFrom(new Vector3D(4, 5, 6, 7))");
var vector_cf2 = new Vector3D();
vector_cf2.copyFrom(new Vector3D(4, 5, 6, 7));
trv(vector_cf2);
trace("")

trace("// new Vector3D(1, 2, 3, 4).copyFrom(new Vector3D(4, 5, 6, 7))");
var vector_cf3 = new Vector3D(1, 2, 3, 4);
vector_cf3.copyFrom(new Vector3D(4, 5, 6, 7));
trv(vector_cf3);
trace("")
trace("")


trace("/// setTo")
trace("// new Vector3D().setTo(6, 7, 8)");
var vector_st = new Vector3D();
vector_st.setTo(6, 7, 8);
trv(vector_st);
trace("")

trace("// new Vector3D(1, 2, 3, 4).setTo(6, 7, 8)");
var vector_st2 = new Vector3D(1, 2, 3, 4);
vector_st2.setTo(6, 7, 8);
trv(vector_st2);
trace("")
trace("")

trace("/// Add");

var vector2 = new Vector3D();
trace("// vector.add(new Vector3D(1, 2, 3, 4))");
trv(vector2.add(new Vector3D(1, 2, 3, 4)));
trace("");

trace("// vector");
trv(vector2);
trace("");

trace("// new Vector3D(5, 6, 8, 9).add(new Vector3D())");
trv(new Vector3D(5, 6, 8, 9).add(new Vector3D()));
trace("");

trace("// new Vector3D(6, -7, 8, -9).add(new Vector3D(-10, 20, -30, 40))");
trv(new Vector3D(6, -7, 8, -9).add(new Vector3D(-10, 20, -30, 40)));
trace("");
trace("");

trace("/// Subtract");

var vector3 = new Vector3D();
trace("// vector.subtract(new Vector3D(1, 2, 3, 4))");
trv(vector3.subtract(new Vector3D(1, 2, 3, 4)));
trace("");

trace("// vector");
trv(vector3);
trace("");

trace("// new Vector3D(5, 6, 8, 9).subtract(new Vector3D())");
trv(new Vector3D(5, 6, 8, 9).subtract(new Vector3D()));
trace("");

trace("// new Vector3D(6, -7, 8, -9).subtract(new Vector3D(-10, 20, -30, 40))");
trv(new Vector3D(6, -7, 8, -9).subtract(new Vector3D(-10, 20, -30, 40)));
trace("");
trace("")


trace("/// incrementBy");

var vector_ib = new Vector3D();
trace("// new Vector3D().incrementBy(new Vector3D())");
vector_ib.incrementBy(new Vector3D());
trv(vector_ib)
trace("");


var vector_ib2 = new Vector3D();
trace("// new Vector3D().incrementBy(new Vector3D(1, 2, -3, 4))");
vector_ib2.incrementBy(new Vector3D(1, 2, -3, 4));
trv(vector_ib2)
trace("");

var vector_ib3 = new Vector3D(3, -4, 5, 6);
trace("// new Vector3D(3, -4, 5, 6).incrementBy(new Vector3D(1, 2, -3, 4))");
vector_ib3.incrementBy(new Vector3D(1, 2, -3, 4));
trv(vector_ib3)
trace("");
trace("");


trace("/// decrementBy");

var vector_db = new Vector3D();
trace("// new Vector3D().decrementBy(new Vector3D())");
vector_db.decrementBy(new Vector3D());
trv(vector_db)
trace("");


var vector_db2 = new Vector3D();
trace("// new Vector3D().decrementBy(new Vector3D(1, 2, -3, 4))");
vector_db2.decrementBy(new Vector3D(1, 2, -3, 4));
trv(vector_db2)
trace("");

var vector_db3 = new Vector3D(3, -4, 5, 6);
trace("// new Vector3D(3, -4, 5, 6).decrementBy(new Vector3D(1, 2, -3, 4))");
vector_db3.decrementBy(new Vector3D(1, 2, -3, 4));
trv(vector_db3)
trace("");
trace("");


trace("/// scaleBy");

trace("// new Vector3D(2, -4, 0, 5).scaleBy(10)");
var vector_sb = new Vector3D(2, -4, 0, 5);
vector_sb.scaleBy(10);
trv(vector_sb);
trace("");

trace("// new Vector3D(2, -4, 0, 5).scaleBy(-0.5)");
var vector_sb2 = new Vector3D(2, -4, 0, 5);
vector_sb2.scaleBy(-0.5);
trv(vector_sb2);
trace("");

trace("// new Vector3D(2, -4, 0, 5).scaleBy(0)");
var vector_sb3 = new Vector3D(2, -4, 0, 5);
vector_sb3.scaleBy(0);
trv(vector_sb3);
trace("");

trace("// new Vector3D(2, -4, 0, 5).scaleBy(1)");
var vector_sb4 = new Vector3D(2, -4, 0, 5);
vector_sb4.scaleBy(1);
trv(vector_sb4);
trace("");

trace("// new Vector3D().scaleBy(100)");
var vector_sb5 = new Vector3D();
vector_sb5.scaleBy(100);
trv(vector_sb5);
trace("");
trace("")


trace("/// negate");

trace("// new Vector3D(2, -4, 0).negate()");
var vector_n = new Vector3D(2, -4, 0);
vector_n.negate();
trv(vector_n);
trace("");

trace("// new Vector3D(2, -4, 0, 5).negate()");
var vector_n2 = new Vector3D(2, -4, 0, 5);
vector_n2.negate();
trv(vector_n2);
trace("");

trace("// new Vector3D().negate()");
var vector_n3 = new Vector3D();
vector_n3.negate();
trv(vector_n3);
trace("");
trace("")


trace("/// Distance");

trace("// Vector3D.distance(new Vector3D(), new Vector3D())");
trace(Vector3D.distance(new Vector3D(), new Vector3D()));
trace("");

trace("// Vector3D.distance(new Point(-100, 200, 300, -400), new Point(100, 200, 300, -400))");
trace(Vector3D.distance(new Vector3D(-100, 200, 300, -400), new Vector3D(100, 200, 300, -400)));
trace("");

trace("// Vector3D.distance(new Point(-100, 200, 300, -400), new Point(-102, 210, 311, -420))");
trace(Vector3D.distance(new Vector3D(-100, 200, 300, -400), new Vector3D(-102, 210, 311, -420)));
trace("");


trace("/// Equals");

var vector4 = new Vector3D();
trace("// vector.equals(new Vector3D(1, 2, 3, 4))");
trace(vector4.equals(new Vector3D(1, 2, 3, 4)));
trace("");

trace("// vector.equals(vector)");
trace(vector4.equals(vector4));
trace("");

trace("// vector");
trv(vector4);
trace("");

trace("// new Vector3D(1, 2, 3).equals(new Vector3D(1, 2, 3, 4))");
trace(new Vector3D(1, 2, 3).equals(new Vector3D(1, 2, 3, 4)));
trace("");
trace("");


trace("/// nearEquals");

trace("// (100, 200, 300).nearEquals((100, 200, 300), 0, false)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 0, false));
trace("// (100, 200, 300).nearEquals((100, 200, 300), 1, false)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 1, false));
trace("// (100, 200, 300).nearEquals((100, 200, 300), 10, false)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 10, false));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 0, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 0, false));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 1, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 1, false));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 10, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 10, false));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 350, 400), 10, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 350, 400), 10, false));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 350, 400), 100, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 350, 400), 100, false));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 450), 10, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 450), 10, false));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 450), 100, false)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 450), 100, false));
trace("");


trace("// (100, 200, 300).nearEquals((100, 200, 300), 0, true)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 0, true));
trace("// (100, 200, 300).nearEquals((100, 200, 300), 1, true)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 1, true));
trace("// (100, 200, 300).nearEquals((100, 200, 300), 10, true)");
trace(new Vector3D(100, 200, 300).nearEquals(new Vector3D(100, 200, 300), 10, true));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 0, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 0, true));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 1, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 1, true));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 400), 10, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 400), 10, true));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 350, 400), 10, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 350, 400), 10, true));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 350, 400), 100, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 350, 400), 100, true));
trace("");

trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 450), 10, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 450), 10, true));
trace("// (100, 200, 300, 400).nearEquals((100, 200, 300, 450), 100, true)");
trace(new Vector3D(100, 200, 300, 400).nearEquals(new Vector3D(100, 200, 300, 450), 100, true));
trace("");
trace("");

trace("// buggy case with allFour=true:")
trace("// (100, 200, 300, 10).nearEquals((100, 200, 300, 20), 100, true)");
trace(new Vector3D(100, 200, 300, 10).nearEquals(new Vector3D(100, 200, 300, 20), 100, true));
trace("// (100, 200, 300, 210).nearEquals((100, 200, 300, 220), 100, true)");
trace(new Vector3D(100, 200, 300, 210).nearEquals(new Vector3D(100, 200, 300, 220), 100, true));
trace("// (100, 200, 300, 0).nearEquals((100, 200, 300, 200), 100, true)");
trace(new Vector3D(100, 200, 300, 0).nearEquals(new Vector3D(100, 200, 300, 200), 100, true));
trace("// (100, 200, 300, 200).nearEquals((100, 200, 300, 0), 100, true)");
trace(new Vector3D(100, 200, 300, 200).nearEquals(new Vector3D(100, 200, 300, 0), 100, true));
trace("// (100, 200, 300, 0).nearEquals((100, 200, 300, -200), 100, true)");
trace(new Vector3D(100, 200, 300, 0).nearEquals(new Vector3D(100, 200, 300, -200), 100, true));
trace("// (100, 200, 300, -200).nearEquals((100, 200, 300, 0), 100, true)");
trace(new Vector3D(100, 200, 300, -200).nearEquals(new Vector3D(100, 200, 300, 0), 100, true));
trace("");
trace("");


trace("/// Clone");

var vector5 = new Vector3D(1, 2, 3, 4);
var clone = vector5.clone();
trace("// vector");
trv(vector5);
trace("");

trace("// clone");
trv(clone);
trace("");

trace("// vector === clone");
trace(vector5 === clone);
trace("");

trace("// vector.equals(clone)");
trace(vector5.equals(clone));
trace("");
trace("");


trace("/// length");
trace("// new Vector3D().length");
trace(new Vector3D().length);
trace("");

trace("// new Vector3D(100, 0).length");
trace(new Vector3D(100, 0).length);
trace("");

trace("// new Vector3D(2, -10, 11, -20).length");
trace(new Vector3D(2, -10, 11, -20).length);
trace("");
trace("");


trace("/// lengthSquared");
trace("// new Vector3D().lengthSquared");
trace(new Vector3D().lengthSquared);
trace("");

trace("// new Vector3D(100, 0).lengthSquared");
trace(new Vector3D(100, 0).lengthSquared);
trace("");

trace("// new Vector3D(100, -200, 300, -400).lengthSquared");
trace(new Vector3D(100, -200, 300, -400).lengthSquared);
trace("");
trace("");


trace("/// Normalize");
trace("// new Vector3D() normalize()");
var vector6 = new Vector3D();
trace(vector6.normalize());
trv(vector6);
trace("");

trace("// new Vector3D(30, 40) normalize()");
var vector7 = new Vector3D(30, 40);
trace(vector7.normalize());
trv(vector7);
trace("");

trace("// new Vector3D(-9, 12, 20) normalize()");
var vector8 = new Vector3D(-9, 12, 20);
trace(vector8.normalize());
trv(vector8);
trace("");

trace("// new Vector3D(-9, 12, 20, -100) normalize()");
var vector9 = new Vector3D(-9, 12, 20, -100);
trace(vector9.normalize());
trv(vector9);
trace("");

trace("// new Vector3D(undefined, 100, 100, 100).normalize()");
var vector10 = new Vector3D(undefined, 100, 100, 100);
trace(vector10.normalize());
trv(vector10);
trace("");
trace("");

trace("// new Vector3D(7, null, 24, 365).normalize()");
var vector11 = new Vector3D(7, null, 24, 365);
trace(vector11.normalize());
trv(vector11);
trace("");
trace("");


trace("/// Project")
trace("// new Vector3D().project()");
var vector_p1 = new Vector3D();
vector_p1.project();
trv(vector_p1);
trace("");

trace("// new Vector3D(1, 2, 3).project()");
var vector_p2 = new Vector3D(1, 2, 3);
vector_p2.project();
trv(vector_p2);
trace("");

trace("// new Vector3D(1, 2, 3, 1).project()");
var vector_p3 = new Vector3D(1, 2, 3, 1);
vector_p3.project();
trv(vector_p3);
trace("");

trace("// new Vector3D(0, 0, 0, 1).project()");
var vector_p4 = new Vector3D(0, 0, 0, 1);
vector_p4.project();
trv(vector_p4);
trace("");

trace("// new Vector3D(20, 30, 40, 10).project()");
var vector_p5 = new Vector3D(20, 30, 40, 10);
vector_p5.project();
trv(vector_p5);
trace("");

trace("// new Vector3D(5, -6, 7, 0.1).project()");
var vector_p6 = new Vector3D(5, -6, 7, 0.1);
vector_p6.project();
trv(vector_p6);
trace("");

trace("// new Vector3D(5, -6, 7, -0.2).project()");
var vector_p7 = new Vector3D(5, -6, 7, -0.2);
vector_p7.project();
trv(vector_p7);
trace("");
trace("");


trace("/// angleBetween")

trace("// Vector3D.angleBetween(new Vector3D(), new Vector3D())");
trace(Vector3D.angleBetween(new Vector3D(), new Vector3D()));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(), new Vector3D(1, 0, 0))");
trace(Vector3D.angleBetween(new Vector3D(), new Vector3D(1, 0, 0)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(1, 0, 0), new Vector3D())");
trace(Vector3D.angleBetween(new Vector3D(1, 0, 0), new Vector3D()));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(1, 0, 0), new Vector3D(0, 1, 0))");
trace(Vector3D.angleBetween(new Vector3D(1, 0, 0), new Vector3D(0, 1, 0)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(0, -1, 0), new Vector3D(0, 0, 1))");
trace(Vector3D.angleBetween(new Vector3D(0, -1, 0), new Vector3D(0, 0, 1)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(0, -20, 0), new Vector3D(0, 0, 0.1))");
trace(Vector3D.angleBetween(new Vector3D(0, -20, 0), new Vector3D(0, 0, 0.1)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(2, 4, 6), new Vector3D(0.6, 0.5, 0.1))");
trace(Vector3D.angleBetween(new Vector3D(2, 4, 6), new Vector3D(0.6, 0.5, 0.1)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(0.6, 0.5, 0.1), new Vector3D(2, 4, 6))");
trace(Vector3D.angleBetween(new Vector3D(0.6, 0.5, 0.1), new Vector3D(2, 4, 6)));
trace("");

trace("// Vector3D.angleBetween(new Vector3D(2, 4, 6, 8), new Vector3D(0.6, 0.5, 0.1, -0.2))");
trace(Vector3D.angleBetween(new Vector3D(2, 4, 6, 8), new Vector3D(0.6, 0.5, 0.1, -0.2)));
trace("");
trace("");


trace("/// dotProduct")

trace("// new Vector3D().dotProduct(new Vector3D())");
trace(new Vector3D().dotProduct(new Vector3D()));
trace("");

trace("// new Vector3D().dotProduct(new Vector3D(1, 0, 0))");
trace(new Vector3D().dotProduct(new Vector3D(1, 0, 0)));
trace("");

trace("// new Vector3D(1, 0, 0).dotProduct(new Vector3D())");
trace(new Vector3D(1, 0, 0).dotProduct(new Vector3D()));
trace("");

trace("// new Vector3D(1, 0, 0).dotProduct(new Vector3D(0, 1, 0))");
trace(new Vector3D(1, 0, 0).dotProduct(new Vector3D(0, 1, 0)));
trace("");

trace("// new Vector3D(0, -1, 0).dotProduct(new Vector3D(0, 0, 1))");
trace(new Vector3D(0, -1, 0).dotProduct(new Vector3D(0, 0, 1)));
trace("");

trace("// new Vector3D(0, -20, 0).dotProduct(new Vector3D(0, 0, 0.1))");
trace(new Vector3D(0, -20, 0).dotProduct(new Vector3D(0, 0, 0.1)));
trace("");

trace("// new Vector3D(2, 4, 6).dotProduct(new Vector3D(0.6, 0.5, 0.1))");
trace(new Vector3D(2, 4, 6).dotProduct(new Vector3D(0.6, 0.5, 0.1)));
trace("");

trace("// new Vector3D(0.6, 0.5, 0.1).dotProduct(new Vector3D(2, 4, 6))");
trace(new Vector3D(0.6, 0.5, 0.1).dotProduct(new Vector3D(2, 4, 6)));
trace("");

trace("// new Vector3D(2, 4, 6, 8).dotProduct(new Vector3D(0.6, 0.5, 0.1, -0.2))");
trace(new Vector3D(2, 4, 6, 8).dotProduct(new Vector3D(0.6, 0.5, 0.1, -0.2)));
trace("");
trace("");


trace("/// crossProduct")

trace("// new Vector3D().crossProduct(new Vector3D())");
trv(new Vector3D().crossProduct(new Vector3D()));
trace("");

trace("// new Vector3D().crossProduct(new Vector3D(1, 0, 0))");
trv(new Vector3D().crossProduct(new Vector3D(1, 0, 0)));
trace("");

trace("// new Vector3D(1, 0, 0).crossProduct(new Vector3D())");
trv(new Vector3D(1, 0, 0).crossProduct(new Vector3D()));
trace("");

trace("// new Vector3D(1, 0, 0).crossProduct(new Vector3D(0, 1, 0))");
trv(new Vector3D(1, 0, 0).crossProduct(new Vector3D(0, 1, 0)));
trace("");

trace("// new Vector3D(0, -1, 0).crossProduct(new Vector3D(0, 0, 1))");
trv(new Vector3D(0, -1, 0).crossProduct(new Vector3D(0, 0, 1)));
trace("");

trace("// new Vector3D(0, -20, 0).crossProduct(new Vector3D(0, 0, 0.1))");
trv(new Vector3D(0, -20, 0).crossProduct(new Vector3D(0, 0, 0.1)));
trace("");

trace("// new Vector3D(2, 4, 6).crossProduct(new Vector3D(0.6, 0.5, 0.1))");
trv(new Vector3D(2, 4, 6).crossProduct(new Vector3D(0.6, 0.5, 0.1)));
trace("");

trace("// new Vector3D(0.6, 0.5, 0.1).crossProduct(new Vector3D(2, 4, 6))");
trv(new Vector3D(0.6, 0.5, 0.1).crossProduct(new Vector3D(2, 4, 6)));
trace("");

trace("// new Vector3D(2, 4, 6, 8).crossProduct(new Vector3D(0.6, 0.5, 0.1, -0.2))");
trv(new Vector3D(2, 4, 6, 8).crossProduct(new Vector3D(0.6, 0.5, 0.1, -0.2)));
trace("");
trace("");
