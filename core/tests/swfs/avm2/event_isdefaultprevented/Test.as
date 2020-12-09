package {
	public class Test {
	}
}

import flash.events.Event;

trace("//var e = new Event(\"test_event\", false, false);");
var e = new Event("test_event", false, false);

trace("//e.isDefaultPrevented();");
trace(e.isDefaultPrevented());

trace("//e.preventDefault()");
e.preventDefault();

trace("//e.isDefaultPrevented();");
trace(e.isDefaultPrevented());

trace("//var e = new Event(\"test_event\", true, true);");
var e = new Event("test_event", true, true);

trace("//e.isDefaultPrevented();");
trace(e.isDefaultPrevented());

trace("//e.preventDefault()");
e.preventDefault();

trace("//e.isDefaultPrevented();");
trace(e.isDefaultPrevented());