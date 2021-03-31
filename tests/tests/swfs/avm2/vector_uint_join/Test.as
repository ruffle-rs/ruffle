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

trace("/// a.join('...');");
trace(a.join("..."));

trace("/// b.join('...');");
trace(b.join("..."));