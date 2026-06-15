var o1 = {};

o1.addProperty("prop", function(p1, p2) {
	trace("o1.prop getter: " + p1 + "," + p2);
	return o1.prop;
}, function(p1, p2) {
	trace("o1.prop setter: " + p1 + "," + p2);
	return o1.prop = "f";
});

o1.prop = "b";

trace("Done: o1.prop: " + o1.prop);

// ==============================================

var o2 = {};

o2.addProperty("prop", function(p1, p2) {
	trace("o2.prop getter: " + p1 + "," + p2);
	o2.prop = "h";
	return "e";
}, function(p1, p2) {
	trace("o2.prop setter: " + p1 + "," + p2);
	return o2.prop;
});

o2.prop = "b";

trace("Done: o2.prop: " + o2.prop);
