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

trace("/// a.indexOf(\"a\");");
trace(a.indexOf("a"));

trace("/// a.indexOf(\"z\");");
trace(a.indexOf("z"));

trace("/// a.indexOf(\"d\");");
trace(a.indexOf("d"));

trace("/// b.indexOf(986);");
trace(b.indexOf(986));

trace("/// b.indexOf(\"986\");");
trace(b.indexOf("986"));

trace("/// b.indexOf(\"Q\");");
trace(b.indexOf("Q"));

trace("/// a.indexOf(\"a\", -5);");
trace(a.indexOf("a", -5));

trace("/// a.indexOf(\"z\", -5);");
trace(a.indexOf("z", -5));

trace("/// a.indexOf(\"d\", -5);");
trace(a.indexOf("d", -5));

trace("/// b.indexOf(986, -5);");
trace(b.indexOf(986, -5));

trace("/// b.indexOf(\"986\", -5);");
trace(b.indexOf("986", -5));

trace("/// b.indexOf(\"Q\", -5);");
trace(b.indexOf("Q", -5));

trace("/// a.indexOf(\"a\", 2);");
trace(a.indexOf("a", 2));

trace("/// a.indexOf(\"z\", 2);");
trace(a.indexOf("z", 2));

trace("/// a.indexOf(\"d\", 2);");
trace(a.indexOf("d", 2));

trace("/// b.indexOf(986, 2);");
trace(b.indexOf(986, 2));

trace("/// b.indexOf(\"986\", 2);");
trace(b.indexOf("986", 2));

trace("/// b.indexOf(\"Q\", 2);");
trace(b.indexOf("Q", 2));