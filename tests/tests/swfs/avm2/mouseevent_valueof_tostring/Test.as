package {
	public class Test {
	}
}
import flash.events.MouseEvent;
import flash.display.MovieClip;

function assert_event(evt: MouseEvent) {
	trace("/// (via Object.prototype.toString...)");
	trace(Object.prototype.toString.call(evt));
	
	trace("/// Object.prototype.valueOf.call(evt) is MouseEvent");
	trace(Object.prototype.valueOf.call(evt) is MouseEvent);
	
	trace("/// (via Object.prototype.valueOf...)");
	trace(Object.prototype.valueOf.call(evt));
	
	trace("/// (via MouseEvent.toString...)");
	trace(evt.toString());
}

trace("/// new MouseEvent('FakeEvent');");
var evt = new MouseEvent('FakeEvent');

assert_event(evt);

trace("/// new MouseEvent(MouseEvent.CLICK, false, true, 50, 50, null, false, false, false, true, 12);");
var evt = new MouseEvent(MouseEvent.CLICK, false, true, 50, 50, null, false, false, false, true, 12);

assert_event(evt);

trace("/// var related_obj = new MovieClip();");
var related_obj = new MovieClip();

trace("/// new MouseEvent(MouseEvent.CLICK, false, true, 202, 103, related_obj, false, false, false, true, 12);");
var evt = new MouseEvent(MouseEvent.CLICK, false, true, 202, 103, related_obj, false, false, false, true, 12);

assert_event(evt);