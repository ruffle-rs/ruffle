package {
	public class Test {
	}
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

trace("//var a = new Array(5);");
var a = new Array(5);

trace("//Array.prototype[0] = 0;");
Array.prototype[0] = 0;

trace("//a[1] = 1;");
a[1] = 1;

trace("//a[2] = 2;");
a[2] = 2;

trace("//a[3] = undefined;");
a[3] = undefined;

trace("//Array.prototype[4] = 4;");
Array.prototype[4] = 4;

trace("//a.length;");
trace(a.length);

trace("//var b = a.reverse();");
var b = a.reverse();

trace("//(contents of a);");
assert_array(a);

trace("//(contents of b);");
assert_array(b);

trace("//Array.prototype[4] = 999;");
Array.prototype[4] = 999;

trace("//(contents of b);");
assert_array(b);