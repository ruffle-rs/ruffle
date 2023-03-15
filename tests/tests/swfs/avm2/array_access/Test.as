package {
	public class Test {
	}
}

var a = new Array("a", "b", "c");

trace(a[0]);
trace(a[1]);
trace(a[2]);

a = new Array(5);
a[0] = "First";
a[2] = "Second";
a[3] = "Third";

trace("a.removeAt(1) = " + a.removeAt(1));
trace("A: " + a);
trace("A.length: " + a.length);

trace("a.removeAt(20) = " + a.removeAt(20));
trace("A: " + a);
trace("A.length: " + a.length);

trace("a.removeAt(-2) = " + a.removeAt(-2));
trace("A: " + a);
trace("A.length: " + a.length);

trace("a.removeAt(-30) = " + a.removeAt(-30));
trace("A: " + a);
trace("A.length: " + a.length);

trace("a.removeAt(0) = " + a.removeAt(0));
trace("A: " + a);
trace("A.length: " + a.length);