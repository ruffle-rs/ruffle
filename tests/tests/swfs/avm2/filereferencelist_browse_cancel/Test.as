package {
    import flash.display.Sprite;
    public class Test extends Sprite { }
}

import flash.net.FileFilter;
import flash.net.FileReferenceList;
import flash.events.Event;

var list: FileReferenceList = new FileReferenceList();
trace("list.fileList: " + list.fileList);

list.addEventListener(Event.SELECT, function (event: Event): void {
    trace("// " + event.type + " event");
});

list.addEventListener(Event.CANCEL, function (event: Event): void {
    trace("// " + event.type + " event");
    trace("event.target: " + event.target);
    trace("list.fileList: " + list.fileList);
});

trace("// browse")
list.browse();
trace("list.fileList: " + list.fileList);
