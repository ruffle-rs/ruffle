package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<int> = new <int>[1,2];");
var a:Vector.<int> = new <int>[1,2];

trace("/// var b: Vector.<int> = new <int>[5,16];");
var b:Vector.<int> = new <int>[5,16];

trace("/// a.map(function (v) { return v * 2; }));");
trace(a.map(function (v) { return v * 2; }));

trace("/// a.map(function (v) { return v * -.5; }));");
trace(a.map(function (v) { return v * -.5; }));

trace("/// b.map(function (v) { return v * 3; }));");
trace(b.map(function (v) { return v * 3; }));

trace("/// b.map(function (v) { return v * -6; }));");
trace(b.map(function (v) { return v * -6; }));