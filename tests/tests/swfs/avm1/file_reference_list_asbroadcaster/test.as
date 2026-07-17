var list = new flash.net.FileReferenceList();

trace(list._listeners.length);

var listener:Object = new Object();
listener.onCancel = function(x) {
    trace("onCancel: " + x);
};
listener.onX = function(x) {
    trace("onX: " + x);
};
list.addListener(listener);

trace(list._listeners.length);
trace(list._listeners[0]);
list._listeners[0].onCancel("a");

trace("Broadcast");

list.broadcastMessage("onCancel", "A");
list.broadcastMessage("onX", "X");
list.broadcastMessage("onY", "Y");

trace("Remove");

list.removeListener(listener);
trace(list._listeners.length);
