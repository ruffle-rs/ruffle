package {
	public class Test {
	}
}

import flash.events.Event;

var e = new Event("test_event");

trace(e.cancelable);

e = new Event("test_event", true, true);

trace(e.cancelable);