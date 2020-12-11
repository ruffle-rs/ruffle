package {
	public class Test {}
}

function testfunc(v1, v2, v3) {
	trace(v1);
	trace(v2);
	trace(v3);
}

trace("///testfunc.apply(null, [\"arg1\", \"arg2\", \"arg3\"]);");
testfunc.apply(null, ["arg1", "arg2", "arg3"]);

trace("///Array.prototype[1] = \"hole\";");
Array.prototype[1] = "hole";

trace("///var a = [];");
var a = [];

trace("///a[2] = \"not a hole\";");
a[2] = "not a hole";

trace("///testfunc.apply(null, a);");
testfunc.apply(null, a);