package {
	public class Test {
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[true, true];");
var b:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a.lastIndexOf(true);");
trace(a.lastIndexOf(true));

trace("/// a.lastIndexOf(false));");
trace(a.lastIndexOf(false));

trace("/// b.lastIndexOf(true);");
trace(b.lastIndexOf(true));

trace("/// b.lastIndexOf(false);");
trace(b.lastIndexOf(false));

trace("/// a.lastIndexOf(true, 1);");
trace(a.lastIndexOf(true, 1));

trace("/// a.lastIndexOf(false, 1));");
trace(a.lastIndexOf(false, 1));

trace("/// b.lastIndexOf(true, 1);");
trace(b.lastIndexOf(true, 1));

trace("/// b.lastIndexOf(false, 1);");
trace(b.lastIndexOf(false, 1));

trace("/// a.lastIndexOf(true, 0);");
trace(a.lastIndexOf(true, 0));

trace("/// a.lastIndexOf(false, 0));");
trace(a.lastIndexOf(false, 0));

trace("/// b.lastIndexOf(true, 0);");
trace(b.lastIndexOf(true, 0));

trace("/// b.lastIndexOf(false, 0);");
trace(b.lastIndexOf(false, 0));