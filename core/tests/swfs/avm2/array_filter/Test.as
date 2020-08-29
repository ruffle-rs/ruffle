package {
	public class Test {
	}
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

trace("//var a = new Array(5,3,1,9,16)");
var a = new Array(5,3,1,9,16);

trace("//var b = a.filter(function (val) { ... });");
var b = a.filter(function (val) {
	return val <= 5;
});

trace("//(contents of b)");
assert_array(b);