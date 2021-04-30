package {
	public class Test {
	}
}

function test_vector() {
	trace("//var a = new Array(8)");
	var a = new Array(8);

	trace("//a[2] = 2");
	a[2] = 2;

	trace("//a[4] = 4");
	a[4] = 4;

	trace("//a[6] = 6");
	a[6] = 6;
	
	return a;
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

trace("//Array.prototype[0] = 999");
Array.prototype[0] = 999;

trace("//Array.prototype[1] = 998");
Array.prototype[1] = 998;

trace("//Array.prototype[3] = 997");
Array.prototype[3] = 997;

trace("//Array.prototype[5] = 996");
Array.prototype[5] = 996;

trace("//Array.prototype[7] = 995");
Array.prototype[7] = 995;

var a = test_vector();

trace("//var b = a.splice();");
var b = a.splice();

trace("//(contents of a)");
assert_array(a);

trace("//b");
trace(b);

var a = test_vector();

trace("//var c = a.splice(0, 3, \"test1\", \"test2\");");
var c = a.splice(0, 3, "test1", "test2");

trace("//(contents of a)");
assert_array(a);

trace("//(contents of c)");
assert_array(c);

var a = test_vector();

trace("//var d = a.splice(-1, 3, \"test3\", \"test4\");");
var d = a.splice(-1, 3, "test3", "test4");

trace("//(contents of a)");
assert_array(a);

trace("//(contents of d)");
assert_array(d);

var a = test_vector();

trace("//var e = a.splice(-3, 3, \"test5\", \"test6\");");
var e = a.splice(-3, 3, "test5", "test6");

trace("//(contents of a)");
assert_array(a);

trace("//(contents of e)");
assert_array(e);

var a = test_vector();

trace("//var e = a.splice(20, 0 \"test7\");");
var e = a.splice(20, 0, "test7");

trace("//(contents of a)");
assert_array(a);

trace("//(contents of e)");
assert_array(e);

var a = test_vector();

trace("//var f = a.splice(2);");
var f = a.splice(2);

trace("//(contents of a)");
assert_array(a);

trace("//(contents of f)");
assert_array(f);

var a = test_vector();

trace("//Array.prototype[0] = 99");
Array.prototype[0] = 99;

trace("//Array.prototype[5] = 96");
Array.prototype[5] = 96;

trace("//Array.prototype[7] = 95");
Array.prototype[7] = 95;

trace("//(contents of a)");
assert_array(a);

trace("//(contents of c)");
assert_array(c);

trace("//(contents of d)");
assert_array(d);

trace("//(contents of e)");
assert_array(e);

trace("//(contents of f)");
assert_array(f);