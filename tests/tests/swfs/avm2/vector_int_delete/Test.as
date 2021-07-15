package {
	public class Test {
	}
}

trace("/// var a: Vector.<int> = new <int>[2,3];");
var a:Vector.<int> = new <int>[2,3];

trace(a[0]);
trace(a[1]);

trace("/// delete a[0];");
trace(delete a[0]);

trace(a[0]);
trace(a[1]);

trace("/// delete a[1];");
trace(delete a[1]);

trace(a[0]);
trace(a[1]);