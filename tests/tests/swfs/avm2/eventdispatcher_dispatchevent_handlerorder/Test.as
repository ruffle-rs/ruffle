package {
	public class Test {
	}
}

import flash.events.Event;
import flash.events.EventDispatcher;

function handler_one(event: Event) {
	trace("//(handler_one executed...)");
}

function handler_two(event: Event) {
	trace("//(handler_two executed...)");
}

function handler_three(event: Event) {
	trace("//(handler_three executed...)");
}

trace("//var evtd = new EventDispatcher();");
var evtd = new EventDispatcher();

trace("//evtd.addEventListener('test', handler_one, false, 0);");
evtd.addEventListener('test', handler_one, false, 0);

trace("//evtd.addEventListener('test', handler_two, false, 5);");
evtd.addEventListener('test', handler_two, false, 5);

trace("//evtd.addEventListener('test', handler_three, false, 0);");
evtd.addEventListener('test', handler_three, false, 0);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));

trace("//evtd.removeEventListener('test', handler_two);");
evtd.removeEventListener('test', handler_two);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));

trace("//evtd.addEventListener('test', handler_two, true, 5);");
evtd.addEventListener('test', handler_two, true, 5);

trace("//evtd.addEventListener('test2', handler_two, false, 5);");
evtd.addEventListener('test2', handler_two, false, 5);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));

trace("//evtd.addEventListener('test', handler_two, false, -5);");
evtd.addEventListener('test', handler_two, false, -5);

trace("//evtd.dispatchEvent('test');");
evtd.dispatchEvent(new Event('test'));