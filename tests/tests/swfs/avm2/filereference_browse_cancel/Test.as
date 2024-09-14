package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;
import flash.events.Event;

var file = new FileReference();

function dump(file) {
    try {
        trace("file.name: " + file.name);
    } catch (e) {
        trace("file.name threw: " + e);
    }
    try {
        trace("file.size: " + file.size);
    } catch (e) {
        trace("file.size threw: " + e);
    }
}

function onselect(e) {
    trace("select event");
    dump(e.target);
}

function oncancel(event) {
    trace("cancel event");
    dump(event.target);
}

file.addEventListener(Event.SELECT, onselect);
file.addEventListener(Event.CANCEL, oncancel);

file.browse();
