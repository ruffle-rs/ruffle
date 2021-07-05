package {
	public class Test {
	}
}

function trace_vector(v: Vector.<*>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[true, true];");
var b:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a.map(function(v) { return v; });");
trace_vector(a.map(function (v) { return v; }));

trace("/// a.map(function(v) { return !v; });");
trace_vector(a.map(function (v) { return !v; }));

trace("/// b.map(function(v) { return v; });");
trace_vector(b.map(function (v) { return v; }));

trace("/// b.map(function(v) { return !v; });");
trace_vector(b.map(function (v) { return !v; }));