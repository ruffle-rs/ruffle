package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<uint> = new <uint>[1,2];");
var a:Vector.<uint> = new <uint>[1,2];

trace("/// var b: Vector.<uint> = new <uint>[5,16];");
var b:Vector.<uint> = new <uint>[5,16];

trace("/// a.every(function (v) { return v > 0; }));");
trace(a.every(function (v) { return v > 0; }));

trace("/// a.every(function (v) { return v > 2; }));");
trace(a.every(function (v) { return v > 2; }));

trace("/// b.every(function (v) { return v > 4; }));");
trace(b.every(function (v) { return v > 4; }));

trace("/// b.every(function (v) { return v > 10; }));");
trace(b.every(function (v) { return v > 10; }));