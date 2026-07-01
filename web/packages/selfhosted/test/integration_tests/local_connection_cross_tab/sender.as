var lc = new LocalConnection();

lc.onStatus = function(info) {
	trace("status:" + info.level);
};

function sendLC(connectionName, methodName) {
	var args = [];
	var i = 2;
	while (i < arguments.length) {
		args.push(arguments[i]);
		i++;
	}

	switch (args.length) {
		case 0:
			lc.send(connectionName, methodName);
			break;
		case 1:
			lc.send(connectionName, methodName, args[0]);
			break;
		case 2:
			lc.send(connectionName, methodName, args[0], args[1]);
			break;
		case 3:
			lc.send(connectionName, methodName, args[0], args[1], args[2]);
			break;
	}
	trace("sent:" + connectionName + ":" + methodName);
}

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

flash.external.ExternalInterface.addCallback("sendLC", null, sendLC);
flash.external.ExternalInterface.addCallback("connectLC", null, connectLC);
flash.external.ExternalInterface.addCallback("disconnectLC", null, disconnectLC);

trace("Hello from Flash!");
