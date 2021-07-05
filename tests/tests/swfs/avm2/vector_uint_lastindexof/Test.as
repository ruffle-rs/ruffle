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

trace("/// a.lastIndexOf(0);");
trace(a.lastIndexOf(0));

trace("/// a.lastIndexOf(1);");
trace(a.lastIndexOf(1));

trace("/// a.lastIndexOf(2);");
trace(a.lastIndexOf(2));

trace("/// b.lastIndexOf(3);");
trace(b.lastIndexOf(3));

trace("/// b.lastIndexOf(5);");
trace(b.lastIndexOf(5));

trace("/// b.lastIndexOf(12);");
trace(b.lastIndexOf(12));

trace("/// a.lastIndexOf(0, 1);");
trace(a.lastIndexOf(0, 1));

trace("/// a.lastIndexOf(1, 1);");
trace(a.lastIndexOf(1, 1));

trace("/// a.lastIndexOf(2, 1);");
trace(a.lastIndexOf(2, 1));

trace("/// b.lastIndexOf(3, 1);");
trace(b.lastIndexOf(3, 1));

trace("/// b.lastIndexOf(5, 1);");
trace(b.lastIndexOf(5, 1));

trace("/// b.lastIndexOf(12, 1);");
trace(b.lastIndexOf(12, 1));

trace("/// a.lastIndexOf(0, -1);");
trace(a.lastIndexOf(0, -1));

trace("/// a.lastIndexOf(1, -1);");
trace(a.lastIndexOf(1, -1));

trace("/// a.lastIndexOf(2, -1);");
trace(a.lastIndexOf(2, -1));

trace("/// b.lastIndexOf(3, -1);");
trace(b.lastIndexOf(3, -1));

trace("/// b.lastIndexOf(5, -1);");
trace(b.lastIndexOf(5, -1));

trace("/// b.lastIndexOf(12, -1);");
trace(b.lastIndexOf(12, -1));