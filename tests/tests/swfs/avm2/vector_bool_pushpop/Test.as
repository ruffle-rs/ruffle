package {
	public class Test {
	}
}

function trace_vector(v: Vector.<*>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(0, \"true\", -1, 3.5, \"false\", false, true);");
trace(a.push(0, "true", -1, 3.5, "false", false, true));

trace("/// (contents of a...)");
trace_vector(a);

trace("/// b.push(0, \"true\", -1, 3.5, \"false\", false, true);");
trace(b.push(0, "true", -1, 3.5, "false", false, true));

trace("/// (contents of b...)");
trace_vector(b);