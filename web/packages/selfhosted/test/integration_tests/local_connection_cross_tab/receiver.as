var lc = new LocalConnection();

lc.test = function() {
	var argStr = "";
	var i = 0;
	while (i < arguments.length) {
		if (i > 0) {
			argStr = argStr + ",";
		}
		argStr = argStr + arguments[i];
		i++;
	}
	trace("received:test:" + arguments.length + " args");
	if (arguments.length > 0) {
		trace("  args=" + argStr);
	}
};

lc.onStatus = function(info) {
	trace("status:" + info.level);
};

function connectLC(name) {
	var result = lc.connect(name);
	if (result) {
		trace("connected:" + name);
		return "ok";
	} else {
		trace("connect_failed:" + name);
		return "error";
	}
}

function disconnectLC() {
	lc.close();
	trace("disconnected");
}

flash.external.ExternalInterface.addCallback("connectLC", null, connectLC);
flash.external.ExternalInterface.addCallback("disconnectLC", null, disconnectLC);

trace("Hello from Flash!");
