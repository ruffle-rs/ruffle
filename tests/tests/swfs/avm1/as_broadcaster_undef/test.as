var broadcaster:Object = new Object();
dumpListeners(" before initialization");

AsBroadcaster.initialize(broadcaster);
dumpListeners(" after initialization");
trace("");

trace("// broadcaster.addListener()");
trace(broadcaster.addListener());
dumpListeners("");
trace("");

trace("// broadcaster.addListener(undefined)");
trace(broadcaster.addListener(undefined));
dumpListeners("");
trace("");

trace("// broadcaster.addListener(null)");
trace(broadcaster.addListener(null));
dumpListeners("");
trace("");

trace("// broadcaster.addListener(false)");
trace(broadcaster.addListener(false));
dumpListeners("");
trace("");

trace("// broadcaster.removeListener(false)");
trace(broadcaster.removeListener(false));
dumpListeners("");
trace("");

trace("// broadcaster.removeListener(false)");
trace(broadcaster.removeListener(false));
dumpListeners("");
trace("");

trace("// broadcaster.removeListener()");
trace(broadcaster.removeListener());
dumpListeners("");
trace("");

trace("// broadcaster.addListener(null)");
trace(broadcaster.addListener(null));
dumpListeners("");
trace("");

trace("// broadcaster.addListener(false)");
trace(broadcaster.addListener(false));
dumpListeners("");
trace("");

trace("// broadcaster.removeListener(undefined)");
trace(broadcaster.removeListener(undefined));
dumpListeners("");
trace("");


trace("// broadcaster._listeners = null");
broadcaster._listeners = [];
trace("");

trace("// broadcaster.addListener()");
trace(broadcaster.addListener());
dumpListeners("");
trace("");

trace("// broadcaster._listeners = []");
broadcaster._listeners = [];
trace("");

var traceListener = function() {
	trace("traceListener was called as a function with " + arguments.length + " args (" + arguments + ")");
};
var valuesToTry = [123, null, undefined, "", " ", "1234", false, true];
function makeListenerFunction(name) {
	traceListener[name] = function() {
		trace("traceListener[\"" + name + "\"] called with " + arguments.length + " args (" + arguments + ")");
	};
}
for (var i in valuesToTry) {
	makeListenerFunction(valuesToTry[i]);
}

trace("// broadcaster.addListener(traceListener)");
trace(broadcaster.addListener(traceListener));
dumpListeners("");
trace("");

trace("// broadcaster.broadcastMessage()");
trace(broadcaster.broadcastMessage());
trace("");

for (var i in valuesToTry) {
	var str = valuesToTry[i];
	if (typeof valuesToTry[i] == "string") {
		str = "\"" + str + "\"";
	}
	trace("// broadcaster.broadcastMessage(" + str + ")");
	trace(broadcaster.broadcastMessage(valuesToTry[i]));
	trace("");
}

function dumpListeners(when) {
	trace("Listeners" + when + ": " + broadcaster._listeners.length + " (" + broadcaster._listeners + ")");
}
