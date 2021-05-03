package {
	public class Test {
	}
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

trace("//var a = new Array(8)");
var a = new Array(8);

trace("//Array.prototype[0] = 999");
Array.prototype[0] = 999;

trace("//Array.prototype[1] = 998");
Array.prototype[1] = 998;

trace("//a[2] = 2");
a[2] = 2;

trace("//Array.prototype[3] = 997");
Array.prototype[3] = 997;

trace("//a[4] = 4");
a[4] = 4;

trace("//Array.prototype[5] = 996");
Array.prototype[5] = 996;

trace("//a[6] = 6");
a[6] = 6;

trace("//Array.prototype[7] = 995");
Array.prototype[7] = 995;

trace("//var b = a.slice();");
var b = a.slice();

trace("//(contents of b)");
assert_array(b);

trace("//var c = a.slice(0, 3);");
var c = a.slice(0, 3);

trace("//(contents of c)");
assert_array(c);

trace("//var d = a.slice(-1, 3);");
var d = a.slice(-1, 3);

trace("//(contents of d)");
assert_array(d);

trace("//var e = a.slice(0, 3);");
var e = a.slice(0, -3);

trace("//(contents of e)");
assert_array(e);

trace("//var f = a.slice(-1, -3);");
var f = a.slice(-1, -3);

trace("//(contents of f)");
assert_array(f);

trace("//var g = a.slice(-3, -1);");
var g = a.slice(-3, -1);

trace("//(contents of g)");
assert_array(g);