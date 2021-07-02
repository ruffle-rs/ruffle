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

trace("/// a.filter(function(v) { return v; });");
trace_vector(a.filter(function (v) { return v; }));

trace("/// a.filter(function(v) { return !v; });");
trace_vector(a.filter(function (v) { return !v; }));

trace("/// b.filter(function(v) { return v; });");
trace_vector(b.filter(function (v) { return v; }));

trace("/// b.filter(function(v) { return !v; });");
trace_vector(b.filter(function (v) { return !v; }));