// Test adding getter/setter properties with `addProperty`.

var foo = {};
foo._bar = 10;
function getBar() {
	return this._bar;
}

function setBar(n) {
	this._bar = n;
}

// Returns false: Invlaid property name.
trace(foo.addProperty(""));

// Return false: Invalid getter.
trace(foo.addProperty("bar"));

// Return false: Invalid setter.
trace(foo.addProperty("bar", getBar));

// Return true.
trace(foo.addProperty("bar", getBar, null));
trace(foo.bar);
foo.bar = 0;
trace(foo.bar);

// Return true.
trace(foo.addProperty("bar", getBar, setBar));
trace(foo.bar);
foo.bar = 0;
trace(foo.bar);

function returnTen() {
	return 10;
}

// TODO: uncomment
/*var o = { a: 3 };
trace(o.addProperty("a", returnTen, null));
trace(o.a);
o.watch("a", function(prop, oldVal, newVal) {
	trace("watcher: " + oldVal + " -> " + newVal);
});
o.a = 4;*/

var o = new Array(3);
trace(o.addProperty("length", returnTen, null));
trace(o.length);
trace(o.push(1));

var o = [1, 2, 3];
trace(o.addProperty(2, returnTen, null));
trace(o[2]);
trace(o.pop());

fscommand("quit");
