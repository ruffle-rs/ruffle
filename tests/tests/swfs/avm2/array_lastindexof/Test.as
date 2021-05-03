package {
	public class Test {
	}
}

trace("//var a = new Array(5,\"5\",3,false,4,5,undefined,9)");
var a = new Array(5,"5",3,false,4,5,undefined,9);

trace("//a.lastIndexOf(5);")
trace(a.lastIndexOf(5));

trace("//a.lastIndexOf(5, 1);")
trace(a.lastIndexOf(5, 1));

trace("//a.lastIndexOf(5, 2);")
trace(a.lastIndexOf(5, 2));

trace("//a.lastIndexOf(5, 6);")
trace(a.lastIndexOf(5, 6));

trace("//a.lastIndexOf(5, 10);")
trace(a.lastIndexOf(5, 10));

trace("//a.lastIndexOf(true);");
trace(a.lastIndexOf(true));

trace("//a.lastIndexOf(undefined);");
trace(a.lastIndexOf(undefined));

trace("//a.lastIndexOf(\"5\");");
trace(a.lastIndexOf("5"));