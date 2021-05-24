package  {
	import flash.display.Sprite;

	public class Test extends Sprite {}
}

// Ensure that Object properties are on the prototype.
var obj:* = Object;
trace("Object.hasOwnProperty('toString'): ", obj.hasOwnProperty('toString'));
trace("Object.prototype.hasOwnProperty('toString'): ", obj.prototype.hasOwnProperty('toString'));

var o: * = {};
trace(o);
Object.prototype.toString = function():String { return "Custom toString"; }
trace(o);

