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

trace("//var d = new Array(\"str\", 123, undefined, null, true, false);");
var d = new Array("str", 123, undefined, null, true, false);

trace("//a.toString();");
trace(a.toString());

trace("//b.toString();");
trace(b.toString());

trace("//c.toString();");
trace(c.toString());

trace("//d.toString();");
trace(d.toString());