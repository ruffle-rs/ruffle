var o = {};

o.prop = "a";

function recurseInfinitely() {
	recurseInfinitely();
}

o.watch("prop", function() {
	trace("watcher");
	recurseInfinitely();
	trace("watcher returning");
	return "b";
});

o.prop = "c";

trace("Done");
trace("  o.prop: " + o.prop);
