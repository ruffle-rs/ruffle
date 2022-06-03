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

trace("//a[1] = \"other_test\";");
a[1] = "other_test";

trace("//a[2] = \"test\";");
a[2] = "test";

trace("//Array.prototype[3] = \"works\";");
Array.prototype[3] = "works";

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);

trace("//a.pop();");
trace(a.pop());

trace("//(contents of a)");
assert_array(a);

trace("//a.length");
trace(a.length);