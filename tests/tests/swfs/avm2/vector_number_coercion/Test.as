package {
	public class Test {
	}
}

trace("/// var a: Vector.<Number> = new <Number>[1,2,3,4];");
var a:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// a[0] = \"5\";");
a[0] = "5";

trace("/// a[1] = \"NaN\";");
a[1] = "NaN";

trace("/// a[2] = -5;");
a[2] = -5;

trace("/// a[3] = true;");
a[3] = true;

trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);