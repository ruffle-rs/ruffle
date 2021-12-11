package {
	public class Test {
	}
}

import flash.events.Event;

trace("//e = new Event(\"test_event\");");
var e = new Event("test_event");

trace("//e.toString()");
trace(e.toString());

trace("//Object.prototype.valueOf.call(e)");
trace(Object.prototype.valueOf.call(e));

trace("//Object.prototype.valueOf.call(e) is Event");
trace(Object.prototype.valueOf.call(e) is Event);

trace("//Object.prototype.toString.call(e)");
trace(Object.prototype.toString.call(e));

trace("//e = new Event(\"test_event\", true, true);");
e = new Event("test_event", true, true);

trace("//e.toString()");
trace(e.toString());

trace("//Object.prototype.valueOf.call(e)");
trace(Object.prototype.valueOf.call(e));

trace("//Object.prototype.valueOf.call(e) is Event");
trace(Object.prototype.valueOf.call(e) is Event);

trace("//Object.prototype.toString.call(e)");
trace(Object.prototype.toString.call(e));