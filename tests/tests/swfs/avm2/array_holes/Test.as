package {
	public class Test {
	}
}

trace("//var a = new Array(5);");
var a = new Array(5);

trace("//Array.prototype[3] = \"works\";");
Array.prototype[3] = "works";

trace("//a[2]");
trace(a[2]);

trace("//a[3]");
trace(a[3]);

trace("//a[3] = \"nohole\"");
a[3] = "nohole";

trace("//a[3]");
trace(a[3]);