package {
	public class Test {
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[true, true];");
var b:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a.indexOf(true);");
trace(a.indexOf(true));

trace("/// a.indexOf(false));");
trace(a.indexOf(false));

trace("/// b.indexOf(true);");
trace(b.indexOf(true));

trace("/// b.indexOf(false);");
trace(b.indexOf(false));