package {
	public class Test {
	}
}

function trace_vector(v: Vector.<int>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<int> = new <int>[1,2];");
var a:Vector.<int> = new <int>[1,2];

trace("/// var b: Vector.<int> = new <int>[5,16];");
var b:Vector.<int> = new <int>[5,16];

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(5);");
trace(a.push(5));

trace("/// (contents of a)...");
trace_vector(a);

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(-15, 32, true, false, \"63\");");
trace(a.push(-15, 32, true, false, "63"));

trace("/// (contents of a)...");
trace_vector(a);

trace("/// b.pop();");
trace(b.pop());

trace("/// b.pop();");
trace(b.pop());

trace("/// b.pop();");
trace(b.pop());