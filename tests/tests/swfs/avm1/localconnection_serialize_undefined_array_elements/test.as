var values = [undefined, "keep"];

trace("--- Source Array ---");
trace("Length: " + values.length);
trace("Has own index 0: " + values.hasOwnProperty("0"));
trace("Index 0 type: " + typeof values[0]);
trace("Index 0: " + values[0]);
trace("Has own index 1: " + values.hasOwnProperty("1"));
trace("Index 1: " + values[1]);

function traceReceivedArray(label, received) {
    trace("--- " + label + " ---");
    trace("Length: " + received.length);

    trace("Has own index 0: " + received.hasOwnProperty("0"));
    trace("Index 0 type: " + typeof received[0]);
    trace("Index 0: " + received[0]);

    trace("Has own index 1: " + received.hasOwnProperty("1"));
    trace("Index 1 type: " + typeof received[1]);
    trace("Index 1: " + received[1]);
}

var receiver = new LocalConnection();

receiver.onReceiveDirect = function(received) {
    traceReceivedArray("Direct Array Result", received);
};

receiver.onReceiveNested = function(payload) {
    traceReceivedArray("Nested Array Result", payload.values);
};

trace("Connected: " + receiver.connect("undefined_array_elements"));

var sender = new LocalConnection();

// Send the same strict array directly and nested in an ordinary object.
sender.send("undefined_array_elements", "onReceiveDirect", values);
sender.send("undefined_array_elements", "onReceiveNested", {
    values: values
});
