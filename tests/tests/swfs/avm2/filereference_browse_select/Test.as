package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;
import flash.net.FileFilter;
import flash.events.Event;

var file = new FileReference();

function dump(file) {
    var properties = ["creationDate", "creator", "data", /* AIR */ "extension", "modificationDate", "name", "size", "type"];

    for each (var property in properties) {
        try {
            trace("file['" + property + "']: " + file[property]);
        } catch (e) {
            trace("file['" + property + "'] throw: " + e);
        }
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

file.browse([new FileFilter("debug-select-success", "*.txt")]);
