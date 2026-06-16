var o1 = {};

o1.prop1 = "a";
o1.prop2 = "t";

o1.watch("prop1", function(p1, p2, p3, p4) {
	trace("o1.prop1 changed");
	trace("  o1.prop1: " + o1.prop1);
	trace("  o1.prop2: " + o1.prop2);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o1.prop2 = "c";
	trace("Set to c, returning d");
	trace("  o1.prop1: " + o1.prop1);
	trace("  o1.prop2: " + o1.prop2);
	return "d";
});

o1.watch("prop2", function(p1, p2, p3, p4) {
	trace("o1.prop2 changed");
	trace("  o1.prop1: " + o1.prop1);
	trace("  o1.prop2: " + o1.prop2);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o1.prop1 = "k";
	trace("Set to k, returning l");
	trace("  o1.prop1: " + o1.prop1);
	trace("  o1.prop2: " + o1.prop2);
	return "l";
});

o1.prop1 = "b";

trace("Done");
trace("  o1.prop1: " + o1.prop1);
trace("  o1.prop2: " + o1.prop2);

// ==================================================

var o2 = {};

o2.addProperty("prop1", function(p1, p2) {
	trace("o2.prop1 getter");
	trace("  p: " + p1 + "," + p2);
	return "e";
}, function(p1, p2) {
	trace("o2.prop1 setter");
	trace("  p: " + p1 + "," + p2);
	return "f";
});

o2.addProperty("prop2", function(p1, p2) {
	trace("o2.prop2 getter");
	trace("  p: " + p1 + "," + p2);
	return "g";
}, function(p1, p2) {
	trace("o2.prop2 setter");
	trace("  p: " + p1 + "," + p2);
	return "h";
});

o2.watch("prop1", function(p1, p2, p3, p4) {
	trace("o2.prop1 changed");
	trace("  o2.prop1: " + o2.prop1);
	trace("  o2.prop2: " + o2.prop2);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o2.prop2 = "c";
	trace("Set to c, returning d");
	trace("  o2.prop1: " + o2.prop1);
	trace("  o2.prop2: " + o2.prop2);
	return "d";
});

o2.watch("prop2", function(p1, p2, p3, p4) {
	trace("o2.prop2 changed");
	trace("  o2.prop1: " + o2.prop1);
	trace("  o2.prop2: " + o2.prop2);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o2.prop1 = "k";
	trace("Set to k, returning l");
	trace("  o2.prop1: " + o2.prop1);
	trace("  o2.prop2: " + o2.prop2);
	return "l";
});

o2.prop1 = "b";

trace("Done");
trace("  o2.prop1: " + o2.prop1);
trace("  o2.prop2: " + o2.prop2);
