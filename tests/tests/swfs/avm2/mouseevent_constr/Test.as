package {
	public class Test {
	}
}
import flash.events.MouseEvent;

function assert_event(evt: MouseEvent) {
	trace("/// evt.type");
	trace(evt.type);

	trace("/// evt.altKey");
	trace(evt.altKey);

	trace("/// evt.buttonDown");
	trace(evt.buttonDown);

	trace("/// evt.ctrlKey");
	trace(evt.ctrlKey);

	trace("/// evt.delta");
	trace(evt.delta);

	trace("/// evt.isRelatedObjectInaccessible");
	trace(evt.isRelatedObjectInaccessible);

	trace("/// evt.localX");
	trace(evt.localX);

	trace("/// evt.localY");
	trace(evt.localY);

	trace("/// evt.movementX");
	trace(evt.movementX);

	trace("/// evt.movementY");
	trace(evt.movementY);

	trace("/// evt.relatedObject");
	trace(evt.relatedObject);

	trace("/// evt.shiftKey");
	trace(evt.shiftKey);

	trace("/// evt.stageX");
	trace(evt.stageX);

	trace("/// evt.stageY");
	trace(evt.stageY);
}

trace("/// var evt = new MouseEvent('FakeEvent');");
var evt = new MouseEvent('FakeEvent');

assert_event(evt);

trace("/// var evt = new MouseEvent(MouseEvent.CLICK, false, true, 50, 50, null, false, false, false, true, 12);");
var evt = new MouseEvent(MouseEvent.CLICK, false, true, 50, 50, null, false, false, false, true, 12);

assert_event(evt);