// Frame 1 script of test.fla (SWF 6, AS1).
// Calls XMLSocket.prototype.connect on a plain object that does not have
// XMLSocket.prototype in its prototype chain.

var obj = new Object();
obj.onConnect = function(success) {
	trace("onConnect: " + success);
	if (success) {
		trace("send returned: " + XMLSocket.prototype.send.call(this, "Hello!"));
	}
};
obj.onData = function(data) {
	trace("onData: " + data);
};
obj.onClose = function() {
	trace("onClose");
};

trace("connect returned: " + XMLSocket.prototype.connect.call(obj, "localhost", 8001));
trace("instanceof XMLSocket: " + (obj instanceof XMLSocket));

stop();
