package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;
import flash.net.FileFilter;
import flash.events.Event;
import flash.events.ProgressEvent;
import flash.utils.setTimeout;

var file = new FileReference();

function dump(file) {
    try {
        trace("file.name: " + file.name);
    } catch (e) {
        trace("file.name threw: " + e);
    }
/* FIXME
    try {
        trace("file.size: " + file.size);
    } catch (e) {
        trace("file.size threw: " + e);
    }
*/
    trace("file.data: " + file.data);
    trace("");
}

function onopen(e) {
    trace("open event");
    dump(e.target);
}

function onprogress(e) {
    trace("progress event");
    trace(e.bytesLoaded + " / " + e.bytesTotal);
    dump(e.target);
}

function oncomplete(e) {
    trace("complete event");
    dump(e.target);

    file.removeEventListener(Event.SELECT, onselect);
    file.removeEventListener(Event.COMPLETE, oncomplete);
    file.addEventListener(Event.SELECT, onselect2);
    file.addEventListener(Event.COMPLETE, oncomplete2);
    file.browse([new FileFilter("debug-select-success", "*.txt")]);
}

function oncomplete2(e) {
    trace("complete event");
    dump(e.target);
}

function onselect(e) {
    trace("select event");
    dump(e.target);
}

function onselect2(e) {
    trace("select event 2");
    dump(e.target);

    file.load();
}

function oncancel(e) {
    trace("cancel event");
    dump(e.target);
}

file.addEventListener(Event.OPEN, onopen);
file.addEventListener(ProgressEvent.PROGRESS, onprogress);
file.addEventListener(Event.COMPLETE, oncomplete);
file.addEventListener(Event.SELECT, onselect);
file.addEventListener(Event.CANCEL, oncancel);

file.save("Hello, World!", "debug-success.txt");
