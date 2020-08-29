package {
	public class Test {
	}
}

trace("//var a = new Array(5,\"5\",3,false,4,5,undefined,9)");
var a = new Array(5,"5",3,false,4,5,undefined,9);

trace("//a.indexOf(5);")
trace(a.indexOf(5));

trace("//a.indexOf(5, 1);")
trace(a.indexOf(5, 1));

trace("//a.indexOf(5, 2);")
trace(a.indexOf(5, 2));

trace("//a.indexOf(5, 6);")
trace(a.indexOf(5, 6));

trace("//a.indexOf(5, 10);")
trace(a.indexOf(5, 10));

trace("//a.indexOf(true);");
trace(a.indexOf(true));

trace("//a.indexOf(undefined);");
trace(a.indexOf(undefined));

trace("//a.indexOf(\"5\");");
trace(a.indexOf("5"));