package {
	public class Test {
	}
}

var a = new Array("a", "b", "c");

trace("//delete a[1]");
trace(delete a[1]);
trace("//Array 0 thru 3...");
trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);
trace("//array.length");
trace(a.length);
trace("//array.hasOwnProperty(1)");
trace(a.hasOwnProperty(1));

trace("//delete a[2]");
trace(delete a[2]);
trace("//Array 0 thru 3...");
trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);
trace("//array.length");
trace(a.length);
trace("//array.hasOwnProperty(2)");
trace(a.hasOwnProperty(2));

trace("//delete a[3]");
trace(delete a[3]);
trace("//Array 0 thru 3...");
trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);
trace("//array.length");
trace(a.length);
trace("//array.hasOwnProperty(3)");
trace(a.hasOwnProperty(3));

trace("//delete a[4]");
trace(delete a[4]);
trace("//Array 0 thru 3...");
trace(a[0]);
trace(a[1]);
trace(a[2]);
trace(a[3]);
trace("//array.length");
trace(a.length);
trace("//array.hasOwnProperty(4)");
trace(a.hasOwnProperty(4));