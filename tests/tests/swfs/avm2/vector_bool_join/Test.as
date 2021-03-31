package {
	public class Test {
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// a.join('...');");
trace(a.join("..."));

trace("/// b.join('...');");
trace(b.join("..."));