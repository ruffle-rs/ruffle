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

trace("/// a.map(function (v) { return v.length; }));");
trace(a.map(function (v) { return v.length; }));

trace("/// a.map(function (v) { return v + \" and\"; }));");
trace(a.map(function (v) { return v + " and"; }));

trace("/// b.map(function (v) { return v.length; }));");
trace(b.map(function (v) { return v.length; }));

trace("/// b.map(function (v) { return v + \"6\"; }));");
trace(b.map(function (v) { return v + "6"; }));