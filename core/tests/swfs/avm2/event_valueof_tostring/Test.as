package {
	public class Test {
	}
}

import flash.events.Event;

var e = new Event("test_event");

trace(Object.prototype.valueOf.call(e));
trace(Object.prototype.toString.call(e));

e = new Event("test_event", true, true);

trace(Object.prototype.valueOf.call(e));
trace(Object.prototype.toString.call(e));