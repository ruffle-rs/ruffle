package {
	public class Test {
	}
}

trace("/// var a: Vector.<int> = new <int>[1,2];");
var a:Vector.<int> = new <int>[1,2];

trace("/// a[0] = \"5\";");
a[0] = "5";

trace("/// a[1] = \"not a number\";");
a[1] = "not a number";

trace(a[0]);
trace(a[1]);