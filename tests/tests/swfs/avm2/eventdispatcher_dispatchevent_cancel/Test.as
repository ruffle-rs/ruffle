package {
import flash.display.Sprite;

public class Test extends Sprite {
}
}

import flash.events.Event;
import flash.events.EventDispatcher;

function nocancel_event(event:Event) {
    trace("//(nocancel_event handler)");
}

function cancel_event(event:Event) {
    trace("//(cancel_event handler)");
    event.preventDefault();
}

function stop_event(event:Event) {
    trace("//(stop_event handler)");
    event.stopPropagation();
}

function stop_immediate_event(event:Event) {
    trace("//(stop_immediate_event handler)");
    event.stopImmediatePropagation();
}

trace("//var evtd = new EventDispatcher();");
var evtd = new EventDispatcher();

trace("//evtd.addEventListener('test', nocancel_event, false, 0);");
evtd.addEventListener('test', nocancel_event, false, 0);

trace("//evtd.dispatchEvent('test');");
trace("dispatchEvent, no cancel: " + evtd.dispatchEvent(new Event('test', true, true)));

trace("//evtd.addEventListener('test', cancel_event, false, 0);");
evtd.addEventListener('test', cancel_event, false, 0);

trace("//evtd.dispatchEvent('test');");
trace("dispatchEvent, cancel: " + evtd.dispatchEvent(new Event('test', true, true)));

trace("//evtd.removeEventListener('test', cancel_event);");
evtd.removeEventListener('test', cancel_event);

trace("//evtd.addEventListener('test', stop_event, false, 5);");
evtd.addEventListener('test', stop_event, false, 5);

trace("//evtd.dispatchEvent('test');");
trace("dispatchEvent, stop: " + evtd.dispatchEvent(new Event('test', true, true)));

trace("//evtd.addEventListener('test', stop_immediate_event, false, 10);");
evtd.addEventListener('test', stop_immediate_event, false, 10);

trace("//evtd.dispatchEvent('test');");
trace("dispatchEvent, stop immediate: " + evtd.dispatchEvent(new Event('test', true, true)));
