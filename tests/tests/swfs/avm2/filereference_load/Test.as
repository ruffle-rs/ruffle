package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;
import flash.net.FileFilter;
import flash.events.Event;
import flash.events.ProgressEvent;

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
}

function onselect(e) {
    trace("select event");
    dump(e.target);

    file.addEventListener(Event.OPEN, onopen);
    file.addEventListener(ProgressEvent.PROGRESS, onprogress);
    file.addEventListener(Event.COMPLETE, oncomplete);

    file.load();
}

file.addEventListener(Event.SELECT, onselect);

file.browse([new FileFilter("debug-select-success", "*.txt")]);
