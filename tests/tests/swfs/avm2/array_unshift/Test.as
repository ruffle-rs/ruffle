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

trace("//a[2] = \"test\";");
a[2] = "test";

trace("//Array.prototype[3] = \"works\";");
Array.prototype[3] = "works";

assert_array(a);

trace("//a.unshift(\"hi\", \"bye\");");
a.unshift("hi", "bye");

assert_array(a);

trace("//a.unshift();");
a.unshift();

assert_array(a);