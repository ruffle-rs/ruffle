package {
	public class Test {
	}
}

import flash.events.EventDispatcher;

trace("//var evtd = new EventDispatcher();");
var evtd = new EventDispatcher();

trace("//var listener = function() { ... };");
var listener = function() { trace("//Listener called!"); };

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.addEventListener('test', listener, false, 0);");
evtd.addEventListener('test', listener, false, 0);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.removeEventListener('test', listener, false);");
evtd.removeEventListener('test', listener, false);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.removeEventListener('test', listener, false);");
evtd.removeEventListener('test', listener, false);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.addEventListener('test', listener, false, 0);");
evtd.addEventListener('test', listener, false, 0);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.addEventListener('test', listener, false, 0);");
evtd.addEventListener('test', listener, false, 0);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.removeEventListener('test', listener, false);");
evtd.removeEventListener('test', listener, false);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));

trace("//evtd.removeEventListener('test', listener, false);");
evtd.removeEventListener('test', listener, false);

trace("//evtd.hasEventListener('test');");
trace(evtd.hasEventListener('test'));