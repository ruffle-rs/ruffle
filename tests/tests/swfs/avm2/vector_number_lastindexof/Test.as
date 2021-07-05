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

trace("/// a.lastIndexOf(0);");
trace(a.lastIndexOf(0));

trace("/// a.lastIndexOf(1);");
trace(a.lastIndexOf(1));

trace("/// a.lastIndexOf(2);");
trace(a.lastIndexOf(2));

trace("/// b.lastIndexOf(3);");
trace(b.lastIndexOf(3));

trace("/// b.lastIndexOf(-5);");
trace(b.lastIndexOf(-5));

trace("/// b.lastIndexOf(NaN);");
trace(b.lastIndexOf(NaN));

trace("/// a.lastIndexOf(0, 1);");
trace(a.lastIndexOf(0, 1));

trace("/// a.lastIndexOf(1, 1);");
trace(a.lastIndexOf(1, 1));

trace("/// a.lastIndexOf(2, 1);");
trace(a.lastIndexOf(2, 1));

trace("/// b.lastIndexOf(3, 1);");
trace(b.lastIndexOf(3, 1));

trace("/// b.lastIndexOf(-5, 1);");
trace(b.lastIndexOf(-5, 1));

trace("/// b.lastIndexOf(NaN, 1);");
trace(b.lastIndexOf(NaN, 1));

trace("/// a.lastIndexOf(0, -2);");
trace(a.lastIndexOf(0, -2));

trace("/// a.lastIndexOf(1, -2);");
trace(a.lastIndexOf(1, -2));

trace("/// a.lastIndexOf(2, -2);");
trace(a.lastIndexOf(2, -2));

trace("/// b.lastIndexOf(3, -2);");
trace(b.lastIndexOf(3, -2));

trace("/// b.lastIndexOf(-5, -2);");
trace(b.lastIndexOf(-5, -2));

trace("/// b.lastIndexOf(NaN, -2);");
trace(b.lastIndexOf(NaN, -2));