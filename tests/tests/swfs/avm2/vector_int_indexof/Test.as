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

trace("/// a.indexOf(0);");
trace(a.indexOf(0));

trace("/// a.indexOf(1);");
trace(a.indexOf(1));

trace("/// a.indexOf(2);");
trace(a.indexOf(2));

trace("/// b.indexOf(3);");
trace(b.indexOf(3));

trace("/// b.indexOf(5);");
trace(b.indexOf(5));

trace("/// b.indexOf(15);");
trace(b.indexOf(15));