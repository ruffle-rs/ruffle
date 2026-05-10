trace("// new Array()");
var a = new Array();
trace(a);
trace(a.length);

trace("// new Array(5)");
var a = new Array(5);
trace(a);
trace(a.length);

trace('// new Array("3")');
var a = new Array("3");
trace(a);
trace(a.length);

trace("// new Array(o)");
var o = { valueOf: function() { return 3; } };
var a = new Array(o);
trace(a);
trace(a.length);

trace("// new Array(-1)");
var a = new Array(-1);
trace(a);
trace(a.length);

trace("// new Array(undefined)");
var a = new Array(undefined);
trace(a);
trace(a.length);

trace("// new Array(null)");
var a = new Array(null);
trace(a);
trace(a.length);

var a = new Array(1, 2, true, 4, 5);
trace("// new Array(1, 2, true, 4, 5)");
trace(a);
trace(a.length);

var a = new Array(undefined, false);
trace("// new Array(undefined, false)");
trace(a);
trace(a.length);

var a = ["a"];
trace('// Array.call(a, "b")');
var b = Array.call(a, "b");
trace(a);
trace(b);

fscommand("quit");
