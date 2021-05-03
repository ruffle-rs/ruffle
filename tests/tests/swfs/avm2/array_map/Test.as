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

trace("//var b = a.map(function (val) { return val + 1; });");
var b = a.map(function (val) {
	return val + 1;
});

trace("//(contents of b)");
assert_array(b);