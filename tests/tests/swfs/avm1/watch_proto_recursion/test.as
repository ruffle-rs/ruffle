var o = {};

o.prop = "a";

var infiniteProto = {};
infiniteProto.__proto__ = infiniteProto;

o.watch("prop", function() {
	trace("watcher");
	infiniteProto.prop;
	trace("watcher returning");
	return "b";
});

o.prop = "c";

trace("Done");
trace("  o.prop: " + o.prop);
