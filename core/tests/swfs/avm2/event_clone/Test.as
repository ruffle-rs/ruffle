package {
	public class Test {
	}
}

import flash.events.Event;

trace("///(DynEvent is a dynamic subclass of Event that fails to override clone)");
dynamic class DynEvent extends Event {
	function DynEvent(type: String, bubbles: Boolean = false, cancelable: Boolean = false) {
		super(type, bubbles, cancelable);
	}
}

trace("///var e = new DynEvent(\"test_event\", false, true);");
var e = new DynEvent("test_event", false, true);

trace("///e.expando = \"Original expando property!\"");
e.expando = "Original expando property!";

trace("///e.expando;");
trace(e.expando);

trace("///var e2 = e.clone();");
var e2 = e.clone();

trace("///e2 === e;");
trace(e2 === e);

trace("///e2.hasOwnProperty('expando');");
trace(e2.hasOwnProperty('expando'));

trace("///e2.type");
trace(e2.type);

trace("///e2.bubbles");
trace(e2.bubbles);

trace("///e2.cancelable");
trace(e2.cancelable);

trace("///e2 instanceof Event");
trace(e2 instanceof Event);

trace("///e2 instanceof DynEvent");
trace(e2 instanceof DynEvent);