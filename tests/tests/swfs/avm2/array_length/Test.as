package {
	public class Test {
	}
}

trace("// [].length;");
trace([].length);
trace("// [0, 1, 2, 3, 4].length;");
trace([0, 1, 2, 3, 4].length);
trace("// [undefined].length;");
trace([undefined].length);

trace("// var a = [0, 1, 2];");
var a = [0, 1, 2];
trace("// a.length;");
trace(a.length);
trace("// a.length = 5;");
a.length = 5;
trace("// a")
trace(a);
trace("// a.length = 0;");
a.length = 0;
trace("// a")
trace(a);
