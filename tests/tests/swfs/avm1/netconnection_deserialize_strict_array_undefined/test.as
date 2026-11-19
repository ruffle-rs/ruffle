trace("--- Starting StrictArray deserialization test ---");

var nc = new NetConnection();

nc.onStatus = function(info) {
    trace("--- NetConnection onStatus ---");
    trace("level: " + info.level);
    trace("code: " + info.code);
};

var responder = new Object();

responder.onResult = function(received) {
    trace("--- Received StrictArray ---");
    trace("Length: " + received.length);

    trace("Has own index 0: " + received.hasOwnProperty("0"));
    trace("Index 0 type: " + typeof received[0]);
    trace("Index 0: " + received[0]);

    trace("Has own index 1: " + received.hasOwnProperty("1"));
    trace("Index 1 type: " + typeof received[1]);
    trace("Index 1: " + received[1]);
};

responder.onStatus = function(info) {
    trace("--- Responder onStatus ---");
    trace("level: " + info.level);
    trace("code: " + info.code);
};

trace("Connecting...");
nc.connect("http://localhost:8000/test1");

trace("Sending NetConnection.call...");
nc.call("test.strictArrayWithUndefined", responder);
trace("NetConnection.call returned");
