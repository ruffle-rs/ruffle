package {
	public class Test {
	}
}

trace("//var a = new Array(\"a\", \"b\", \"c\");");
var a = new Array("a", "b", "c");

trace("//var b = new Array(1, 2, 3);");
var b = new Array(1, 2, 3);

trace("//var c = new Array(a, b);");
var c = new Array(a, b);

trace("//a.toLocaleString();");
trace(a.toLocaleString());

trace("//b.toLocaleString();");
trace(b.toLocaleString());

trace("//c.toLocaleString();");
trace(c.toLocaleString());