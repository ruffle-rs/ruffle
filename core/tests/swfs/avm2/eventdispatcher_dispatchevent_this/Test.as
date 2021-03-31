package {
	public class Test {
	}
}

import flash.events.Event;
import flash.events.EventDispatcher;

function introspect_this(event: Event) {
	trace("//this");
	trace(this);
}

trace("//var evtd = new EventDispatcher();");
var evtd = new EventDispatcher();

trace("//evtd.addEventListener('test', introspect_this, false, 0);");
evtd.addEventListener('test', introspect_this, false, 0);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));