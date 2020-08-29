package {
	public class Test {
	}
}

trace("//var a = new Array(5,\"abc\")");
var a = new Array(5,"abc");

trace("//a.forEach(function (val) { ... }, a);");
a.forEach(function (val, index, array) {
	trace("//(in callback) this === a;")
	trace(this === a);
	trace("//val");
	trace(val);
	trace("//index");
	trace(index);
	trace("//array === a");
	trace(array === a);
}, a);