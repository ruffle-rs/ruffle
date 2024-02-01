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
    trace("event.target: " + event.target);
    trace("list.fileList: " + list.fileList);
    trace("list.fileList[0].name: " + list.fileList[0].name);
});

list.addEventListener(Event.CANCEL, function (event: Event): void {
    trace("// " + event.type + " event");
});

trace("// browse")
list.browse([new FileFilter("debug-select-success", "*.txt")]);
trace("list.fileList: " + list.fileList);
