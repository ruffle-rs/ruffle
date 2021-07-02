package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a.filter(function (v) { return v.length > 0; }));");
trace(a.filter(function (v) { return v.length > 0; }));

trace("/// a.filter(function (v) { return v.length > 2; }));");
trace(a.filter(function (v) { return v.length > 2; }));

trace("/// b.filter(function (v) { return v.length > 0; }));");
trace(b.filter(function (v) { return v.length > 0; }));

trace("/// b.filter(function (v) { return v.length > 4; }));");
trace(b.filter(function (v) { return v.length > 4; }));