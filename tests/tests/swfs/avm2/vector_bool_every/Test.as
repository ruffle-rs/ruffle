package {
	public class Test {
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[true, true];");
var b:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a.every(function(v) { return v; });");
trace(a.every(function (v) { return v; }));

trace("/// a.every(function(v) { return true; });");
trace(a.every(function (v) { return true; }));

trace("/// b.every(function(v) { return v; });");
trace(b.every(function (v) { return v; }));

trace("/// b.every(function(v) { return true; });");
trace(b.every(function (v) { return true; }));