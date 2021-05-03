package {
	public class Test {
	}
}

import flash.events.Event;
import flash.events.EventDispatcher;

function introspect_event(event: Event) {
	trace("//(Handled an event...)");
	
	trace("//event.type");
	trace(event.type);
	
	trace("//event.eventPhase");
	trace(event.eventPhase);
	
	trace("//event.target === evtd");
	trace(event.target === evtd);
	
	trace("//event.currentTarget === evtd");
	trace(event.currentTarget === evtd);
}

trace("//var evtd = new EventDispatcher();");
var evtd = new EventDispatcher();

trace("//evtd.addEventListener('test', introspect_event, false, 0);");
evtd.addEventListener('test', introspect_event, false, 0);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));