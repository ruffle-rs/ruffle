package {
	public class Test {
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[1,2,3,4];");
var a:Vector.<Boolean> = new <Boolean>[1,2,3,4];

trace("/// a[0] = 1;");
a[0] = 1;

trace("/// a[1] = NaN;");
a[1] = NaN;

trace("/// a[2] = \"false\";");
a[2] = "false";

trace("/// a[3] = true;");
a[3] = true;

trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);