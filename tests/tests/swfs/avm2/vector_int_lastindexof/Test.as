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

trace("/// b.lastIndexOf(15);");
trace(b.lastIndexOf(15));

trace("/// a.lastIndexOf(0, 0);");
trace(a.lastIndexOf(0, 0));

trace("/// a.lastIndexOf(1, 0);");
trace(a.lastIndexOf(1, 0));

trace("/// a.lastIndexOf(2, 0);");
trace(a.lastIndexOf(2, 0));

trace("/// b.lastIndexOf(3, 0);");
trace(b.lastIndexOf(3, 0));

trace("/// b.lastIndexOf(5, 0);");
trace(b.lastIndexOf(5, 0));

trace("/// b.lastIndexOf(15, 0);");
trace(b.lastIndexOf(15, 0));

trace("/// a.lastIndexOf(0, -2);");
trace(a.lastIndexOf(0, -2));

trace("/// a.lastIndexOf(1, -2);");
trace(a.lastIndexOf(1, -2));

trace("/// a.lastIndexOf(2, -2);");
trace(a.lastIndexOf(2, -2));

trace("/// b.lastIndexOf(3, -2);");
trace(b.lastIndexOf(3, -2));

trace("/// b.lastIndexOf(5, -2);");
trace(b.lastIndexOf(5, -2));

trace("/// b.lastIndexOf(15, -2);");
trace(b.lastIndexOf(15, -2));