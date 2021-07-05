package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Number> = new <Number>[1,2,3,4];");
var a:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a.map(function (v) { return \"6\"; }));");
trace(a.map(function (v) { return "6"; }));

trace("/// a.map(function (v) { return v * 6; }));");
trace(a.map(function (v) { return v * 6; }));

trace("/// b.map(function (v) { return v * -1; }));");
trace(b.map(function (v) { return v * -1; }));

trace("/// b.map(function (v) { return v * -2; }));");
trace(b.map(function (v) { return v * -2; }));

trace("/// b.map(function (v) { return v * -6; }));");
trace(b.map(function (v) { return v * -6; }));