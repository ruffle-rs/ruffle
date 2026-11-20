// This test instruments ASnative to catch when the FP playerglobal
// calls its various internal helper functions.

var realASnative = ASnative;
ASnative = function(i, j) {
	var name = "CALL: ASnative(" + i + ", " + j + ")";
	trace(name);
	var native = realASnative(i, j);
	return function() {
		trace(name + "(" + arguments.join(", ") + ")");
		var res = native.apply(this, arguments);
		trace("---> " + res + " (" + (typeof res) + ")");
	};
};


trace('// new NetConnection("foo")');
var netconn = new NetConnection("foo");

netconn.call = function(method, callbacks) {
	trace("CALL: netconn.call(" + method + ", " + callbacks + ")");
	trace("> callbacks.onResult(42)");
	callbacks.onResult(42);
};


trace("");
trace("// new NetStream(instrumented netconn)");
var netstream = new NetStream(netconn);

trace('// netstream.publish()');
netstream.publish();
trace('// netstream.publish("fake.flv")');
netstream.publish("fake.flv");
trace('// netstream.publish("fake.flv", "live")');
netstream.publish("hello", "live");

trace('// netstream.play()');
netstream.play();
trace('// netstream.play("fake.flv")');
netstream.play("fake.flv");
trace('// netstream.play("fake.flv", 5)');
netstream.play("fake.flv", 5);
trace('// netstream.play("fake.flv", 5, 6)');
netstream.play("fake.flv", 5, 6);
trace('// netstream.play("fake.flv", 5, 6, "resetObj", "foo")');
netstream.play("fake.flv", 5, 6, "resetObj", "foo");

trace('// netstream.play2()');
netstream.play2();
netstream.play2("playParamObj");

trace('// netstream.receiveAudio()');
netstream.receiveAudio();
trace('// netstream.receiveVideo()');
netstream.receiveVideo();

trace('// netstream.pause()');
netstream.pause();

trace('// netstream.seek()');
netstream.seek();
trace('// netstream.seek(42)');
netstream.seek(42);


trace("");
trace('// Camera.get()');
Camera["get"]();
trace('// Camera.get(999)');
Camera["get"](999);
trace('// Microphone.get()');
Microphone["get"]();
trace('// Microphone.get(999)');
Microphone["get"](999);


trace("");
trace('// SharedObject.getLocal()');
SharedObject.getLocal();
trace('// SharedObject.getLocal("nonexistent", "/")');
SharedObject.getLocal("nonexistent", "/");
trace('// SharedObject.getLocal("nonexistent", "/", true, "foo")');
SharedObject.getLocal("nonexistent", "/", true, "foo");

trace('// SharedObject.getRemote()');
SharedObject.getRemote();
trace('// SharedObject.getRemote("nonexistent", "/")');
SharedObject.getRemote("nonexistent", "/");
trace('// SharedObject.getRemote("nonexistent", "/", "obj", true, "foo")');

SharedObject.getRemote("nonexistent", "/", "obj", true, "foo");
trace('// SharedObject.deleteAll("nonexistent", "foo")');
SharedObject.deleteAll("nonexistent", "foo", "foo");
trace('// SharedObject.getDiskUsage("nonexistent", "foo")');
SharedObject.getDiskUsage("nonexistent", "foo");


trace("");
trace('// new System.Product("hello")');
var product = new System.Product("hello");
trace('// product.isRunning("foo")');
product.isRunning("foo");
trace('// product.isInstalled("foo")');
product.isInstalled("foo");
// TODO: test more args combinations?
trace('// product.launch("foo")');
product.launch("foo");
// TODO: test more args combinations?
trace('// product.download("foo")');
product.download("foo");
trace('// product.installedVersion("foo")');
product.installedVersion("foo");


trace("");
trace('// RemoteLSOUsage.getURLPageSupport("foo")');
RemoteLSOUsage.getURLPageSupport("foo");


trace("");
trace('// new flash.net.FileReference("foo")');
new flash.net.FileReference("foo");

fscommand("quit");
