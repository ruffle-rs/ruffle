package {
	public class Test {
	}
}

import flash.events.EventDispatcher;

trace("///Object.prototype.toString = function() { /* ... */ };");
Object.prototype.toString = (function (old_ts) {
	return function () {
		trace("///(Object.prototype.toString called)");
		return old_ts.apply(this, arguments);
	}
}(Object.prototype.toString));

trace("///var ed = new EventDispatcher();");
var ed = new EventDispatcher();

trace("///ed.toString();");
trace(ed.toString());

class CustomDispatch extends EventDispatcher {
	public override function toString() : String {
		trace("///(CustomDispatch.prototype.toString called);");
		
		return super.toString();
	}
}

trace("///var cust = new CustomDispatch();");
var cust = new CustomDispatch();

trace("///cust.toString();");
trace(cust.toString());