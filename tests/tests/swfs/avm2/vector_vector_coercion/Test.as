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

trace("/// var c: Vector.<Vector.<int>> = new <Vector.<int>>[];");
var c:Vector.<Vector.<int>> = new <Vector.<int>>[];

trace("/// c[0] = a;");
c[0] = a;

trace("/// c[1] = b;");
c[1] = b;

trace("/// (contents of c...)");
trace_vec(c);