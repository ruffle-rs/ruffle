package {
	public class Test {
	}
}

function trace_vector(v: Vector.<String>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(123, {}, \"abc\", true, false);");
trace(a.push(123, {}, "abc", true, false));

trace("/// (contents of a...)");
trace_vector(a);

trace("/// b.pop();");
trace(b.pop());

trace("/// b.push(NaN, -83.5);");
trace(b.push(NaN, -83.5));

trace("/// (contents of b...)");
trace_vector(b);