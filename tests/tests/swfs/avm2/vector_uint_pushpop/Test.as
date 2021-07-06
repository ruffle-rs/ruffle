package {
	public class Test {
	}
}

function trace_vector(v: Vector.<uint>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<uint> = new <uint>[1,2];");
var a:Vector.<uint> = new <uint>[1,2];

trace("/// var b: Vector.<uint> = new <uint>[5,16];");
var b:Vector.<uint> = new <uint>[5,16];

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(0, -1, -2.5, \"16\", \"NaN\");");
trace(a.push(0, -1, -2.5, "16", "NaN"));

trace("/// (contents of a...)");
trace_vector(a);

trace("/// b.pop();");
trace(b.pop());

trace("/// b.push(true, 15.23, \"true\");");
trace(b.push(true, 15.23, "true"));

trace("/// (contents of b...)");
trace_vector(b);