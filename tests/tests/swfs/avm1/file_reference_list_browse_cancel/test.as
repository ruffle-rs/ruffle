var list = new flash.net.FileReferenceList();
trace("fileList = " + list.fileList);

var listener = new Object();
listener.onSelect = function(target) {
    trace("onSelect");
};
listener.onCancel = function(target) {
    trace("onCancel");
    trace("target.fileList = " + target.fileList);
};
list.addListener(listener);

var result = list.browse();
trace("browse returned " + result);
