var a = new flash.net.FileReferenceList();
var b = new flash.net.FileReferenceList();
trace("list own _listeners: " + a.hasOwnProperty("_listeners"));
trace("list shared: " + (a._listeners == b._listeners));
trace("list proto _listeners: " + flash.net.FileReferenceList.prototype._listeners);
var la = {};
la.onSelect = function() { trace("la.onSelect"); };
var lb = {};
lb.onSelect = function() { trace("lb.onSelect"); };
a.addListener(la);
trace("a len: " + a._listeners.length + ", b len: " + b._listeners.length);
b.addListener(lb);
trace("a len: " + a._listeners.length + ", b len: " + b._listeners.length);
a.broadcastMessage("onSelect");

var c = new flash.net.FileReference();
var d = new flash.net.FileReference();
trace("ref own _listeners: " + c.hasOwnProperty("_listeners"));
trace("ref shared: " + (c._listeners == d._listeners));
c.addListener(la);
trace("c len: " + c._listeners.length + ", d len: " + d._listeners.length);
trace("ref shares with list: " + (c._listeners == a._listeners));


trace("ref _listeners enumerable: " + c.isPropertyEnumerable("_listeners"));
trace("list _listeners enumerable: " + a.isPropertyEnumerable("_listeners"));
var protoListeners = flash.net.FileReference.prototype._listeners;
trace("delete ref _listeners: " + delete c._listeners);
trace("ref falls back to the prototype's _listeners: " + (c._listeners == protoListeners));

var lc = {};
lc.onSelect = function() { trace("lc.onSelect"); };
trace("addListener after delete: " + c.addListener(lc));
trace("ref own _listeners after addListener: " + c.hasOwnProperty("_listeners"));
trace("proto _listeners len: " + protoListeners.length);
trace("d len: " + d._listeners.length);
var e = new flash.net.FileReference();
trace("new ref len: " + e._listeners.length);
c.broadcastMessage("onSelect");
d.broadcastMessage("onSelect");
