var o1 = {};

o1.prop = "a";

o1.watch("prop", function(p1, p2, p3, p4) {
	trace("o1.prop changed");
	trace("  o1.prop: " + o1.prop);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o1.prop = "c";
	trace("Set to c, returning d");
	trace("  o1.prop: " + o1.prop);
	return "d";
});

o1.prop = "b";

trace("Done");
trace("  o1.prop: " + o1.prop);

// =============================================

var o2 = {};

o2.addProperty("prop", function(p1, p2) {
	trace("o2.prop getter");
	trace("  p: " + p1 + "," + p2);
	return "e";
}, function(p1, p2) {
	trace("o2.prop setter");
	trace("  p: " + p1 + "," + p2);
	return "f";
});

o2.watch("prop", function(p1, p2, p3, p4) {
	trace("o2.prop changed");
	trace("  o2.prop: " + o2.prop);
	trace("  p: " + p1 + "," + p2 + "," + p3 + "," + p4);
	o2.prop = "c";
	trace("Set to c, returning d");
	trace("  o2.prop: " + o2.prop);
	return "d";
});

o2.prop = "b";

trace("Done");
trace("  o2.prop: " + o2.prop);
