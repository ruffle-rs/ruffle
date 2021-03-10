package {
	public class Test {
	}
}

trace("/// var a: Vector.<uint> = new <uint>[1,2,3,4];");
var a:Vector.<uint> = new <uint>[1,2,3,4];

trace("/// a[0] = \"5\";");
a[0] = "5";

trace("/// a[1] = \"not a number\";");
a[1] = "not a number";

trace("/// a[2] = -5;");
a[2] = -5;

trace("/// a[3] = false;");
a[3] = false;

trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);