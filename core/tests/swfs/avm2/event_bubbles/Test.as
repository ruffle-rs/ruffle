package {
	public class Test {
	}
}

import flash.events.Event;

var e = new Event("test_event");

trace(e.bubbles);

e = new Event("test_event", true, true);

trace(e.bubbles);