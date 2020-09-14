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

trace("//a.join();");
trace(a.join());

trace("//b.join();");
trace(b.join());

trace("//c.join();");
trace(c.join());

trace("//c.join(undefined);");
trace(c.join(undefined));

trace("//c.join(null);");
trace(c.join(null));

trace("//c.join(false);");
trace(c.join(false));

trace("//a.join(NaN);");
trace(a.join(NaN));

trace("//b.join(5);");
trace(b.join(5));

trace("//c.join(\" + \");");
trace(c.join(" + "));

trace("//c.join(b);");
trace(c.join(b));

trace("//d.join(\"!\");");
trace(d.join("!"));