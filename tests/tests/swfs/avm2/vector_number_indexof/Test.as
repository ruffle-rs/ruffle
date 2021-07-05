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

trace("/// a.indexOf(0);");
trace(a.indexOf(0));

trace("/// a.indexOf(1);");
trace(a.indexOf(1));

trace("/// a.indexOf(2);");
trace(a.indexOf(2));

trace("/// b.indexOf(3);");
trace(b.indexOf(3));

trace("/// b.indexOf(-5);");
trace(b.indexOf(-5));

trace("/// b.indexOf(NaN);");
trace(b.indexOf(NaN));

trace("/// a.indexOf(0, 2);");
trace(a.indexOf(0, 2));

trace("/// a.indexOf(1, 2);");
trace(a.indexOf(1, 2));

trace("/// a.indexOf(2, 2);");
trace(a.indexOf(2, 2));

trace("/// b.indexOf(3, 2);");
trace(b.indexOf(3, 2));

trace("/// b.indexOf(-5, 2);");
trace(b.indexOf(-5, 2));

trace("/// b.indexOf(NaN, 2);");
trace(b.indexOf(NaN, 2));

trace("/// a.indexOf(0, -1);");
trace(a.indexOf(0, -1));

trace("/// a.indexOf(1, -1);");
trace(a.indexOf(1, -1));

trace("/// a.indexOf(2, -1);");
trace(a.indexOf(2, -1));

trace("/// b.indexOf(3, -1);");
trace(b.indexOf(3, -1));

trace("/// b.indexOf(-5, -1);");
trace(b.indexOf(-5, -1));

trace("/// b.indexOf(NaN, -1);");
trace(b.indexOf(NaN, -1));