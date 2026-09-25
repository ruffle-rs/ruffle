var o1 = {};

o1.addProperty("prop1", function(p1, p2) {
	trace("o1.prop1 getter: " + p1 + "," + p2);
	return o1.prop2;
}, function(p1, p2) {
	trace("o1.prop1 setter: " + p1 + "," + p2);
	return o1.prop2 = "f";
});

o1.addProperty("prop2", function(p1, p2) {
	trace("o1.prop2 getter: " + p1 + "," + p2);
	return o1.prop1;
}, function(p1, p2) {
	trace("o1.prop2 setter: " + p1 + "," + p2);
	return o1.prop1 = "p";
});

o1.prop1 = "b";
o1.prop2 = "h";

trace("Done");
trace("  o1.prop1: " + o1.prop1);
trace("  o1.prop2: " + o1.prop2);
