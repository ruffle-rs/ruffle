package {
	public class Test {
	}
}

function trace_vector(v) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		if (v[i] is Vector.<int>) {
			trace("/// (contents of index", i, ")");
			trace_vector(v[i]);
		} else {
			trace(v[i]);
		}
	}
}

trace("/// var a:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// (contents of a.pop()...)");
trace_vector(a.pop());

trace("/// (contents of a.pop()...)");
trace_vector(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(new <int>[15,9], new <int>[-1,-94], new <int>[2], new <int>[16]);");
trace(a.push(new <int>[15,9], new <int>[-1,-94], new <int>[2], new <int>[16]));

trace("/// (contents of a...)");
trace_vector(a);

trace("/// (contents of b.pop()...)");
trace_vector(b.pop());

trace("/// b.push(new <int>[-1,-94]);");
trace(b.push(new <int>[-16,-4]));

trace("/// (contents of b...)");
trace_vector(b);

trace("/// b.length = 6;");
trace(b.length = 6);

trace("/// b.pop()");
trace(b.pop());

trace("/// (contents of b...)");
trace_vector(b);