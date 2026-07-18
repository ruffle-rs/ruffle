var list = new flash.net.FileReferenceList();
trace("fileList = " + list.fileList);

var listener = new Object();
listener.onSelect = function(target) {
    trace("onSelect");
    trace("target.fileList.length = " + target.fileList.length);
    for (var i = 0; i < target.fileList.length; i++) {
        trace("name[" + i + "] = " + target.fileList[i].name);
    }
};
listener.onCancel = function(target) {
    trace("onCancel");
};
list.addListener(listener);

var filter = new Object();
filter.description = "debug-select-success";
filter.extension = "*.txt";

var result = list.browse([filter]);
trace("browse returned " + result);
