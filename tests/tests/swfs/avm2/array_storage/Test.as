package {
	public class Test {
	}
}

var a = new Array("a", "b", "c");

trace("//Overwriting array values 0 thru 3");
a[0] = "d";
a[1] = "e";
a[2] = "f";
a[3] = "g";

trace("//Array 0 thru 3...");
trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);
trace("//array.length");
trace(a.length);