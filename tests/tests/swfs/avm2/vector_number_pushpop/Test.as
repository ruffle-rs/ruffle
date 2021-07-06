package {
	public class Test {
	}
}

function trace_vector(v: Vector.<Number>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Number> = new <Number>[1,2,3,4];");
var a:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(-16, 3.2, 5, \"test\", true, false);");
trace(a.push(-16, 3.2, 5, "test", true, false));

trace("/// (contents of a...)");
trace_vector(a);

trace("/// b.pop();");
trace(b.pop());

trace("/// b.pop();");
trace(b.pop());

trace("/// b.push(NaN, \"NaN\", 0);");
trace(b.push(NaN, "NaN", 0));

trace("/// (contents of b...)");
trace_vector(b);