// Frame 1 script of test.fla (SWF 6, AS1).
// Mirrors the FFDK socket library used by the CokeMusic games:
// XMLSocket is "subclassed" via __proto__ and the XMLSocket constructor
// is never run on the instance.

var Sock = function(name) {
	this._name = name;
	this._status = "closed";
};
Sock.prototype.__proto__ = XMLSocket.prototype;
Sock.prototype.connectXMLSocket = XMLSocket.prototype.connect;
Sock.prototype.closeXMLSocket = XMLSocket.prototype.close;
Sock.prototype.connect = function(host, port) {
	this._status = "opening";
	var result = this.connectXMLSocket(host, port);
	trace("connectXMLSocket returned: " + result);
	if (!result) {
		this._status = "closed";
	}
	return result;
};
Sock.prototype.onConnect = function(success) {
	trace("onConnect: " + success);
	if (success) {
		this._status = "opened";
		this.send("Hello!");
	}
};
Sock.prototype.onXML = function(xml) {
	trace("onXML: " + xml);
};
Sock.prototype.onClose = function() {
	trace("onClose");
	this._status = "closed";
};

var sock = new Sock("sock");
trace("instanceof XMLSocket: " + (sock instanceof XMLSocket));
trace("send before connect: " + sock.send("ignored"));
trace("close before connect: " + sock.closeXMLSocket());
trace("connect returned: " + sock.connect("localhost", 8001));
trace("status: " + sock._status);

stop();
