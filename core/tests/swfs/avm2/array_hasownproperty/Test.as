package {
	public class Test {
	}
}

trace("//var a = new Array(5);");
var a = new Array(5);

trace("//Array.prototype[3] = \"works\";");
Array.prototype[3] = "works";

trace("//a.hasOwnProperty(\"2\")");
trace(a.hasOwnProperty("2"));

trace("//a.hasOwnProperty(\"3\")");
trace(a.hasOwnProperty("3"));

trace("//a[3] = \"nohole\"");
a[3] = "nohole";

trace("//a.hasOwnProperty(\"2\")");
trace(a.hasOwnProperty("2"));

trace("//a.hasOwnProperty(\"3\")");
trace(a.hasOwnProperty("3"));