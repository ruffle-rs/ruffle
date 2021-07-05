package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a.lastIndexOf(\"a\");");
trace(a.lastIndexOf("a"));

trace("/// a.lastIndexOf(\"z\");");
trace(a.lastIndexOf("z"));

trace("/// a.lastIndexOf(\"d\");");
trace(a.lastIndexOf("d"));

trace("/// b.lastIndexOf(986);");
trace(b.lastIndexOf(986));

trace("/// b.lastIndexOf(\"986\");");
trace(b.lastIndexOf("986"));

trace("/// b.lastIndexOf(\"Q\");");
trace(b.lastIndexOf("Q"));

trace("/// a.lastIndexOf(\"a\", -2);");
trace(a.lastIndexOf("a", -2));

trace("/// a.lastIndexOf(\"z\", -2);");
trace(a.lastIndexOf("z", -2));

trace("/// a.lastIndexOf(\"d\", -2);");
trace(a.lastIndexOf("d", -2));

trace("/// b.lastIndexOf(986, -2);");
trace(b.lastIndexOf(986, -2));

trace("/// b.lastIndexOf(\"986\", -2);");
trace(b.lastIndexOf("986", -2));

trace("/// b.lastIndexOf(\"Q\", -2);");
trace(b.lastIndexOf("Q", -2));

trace("/// a.lastIndexOf(\"a\", 2);");
trace(a.lastIndexOf("a", 2));

trace("/// a.lastIndexOf(\"z\", 2);");
trace(a.lastIndexOf("z", 2));

trace("/// a.lastIndexOf(\"d\", 2);");
trace(a.lastIndexOf("d", 2));

trace("/// b.lastIndexOf(986, 2);");
trace(b.lastIndexOf(986, 2));

trace("/// b.lastIndexOf(\"986\", 2);");
trace(b.lastIndexOf("986", 2));

trace("/// b.lastIndexOf(\"Q\", 2);");
trace(b.lastIndexOf("Q", 2));