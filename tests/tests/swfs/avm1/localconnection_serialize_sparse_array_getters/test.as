var getterCalls = 0;

var values = new Array();
values.length = 3;
values[2] = "own_two";

// An enumerable getter at index 1, with index 0 left sparse.
values.addProperty("1", function() {
    getterCalls++;
    trace("Array getter invoked");
    return "getter_value";
}, null);

// Verify that index 1 is enumerable without invoking its getter.
var getterIsEnumerable = false;
for (var key in values) {
    if (key == "1") {
        getterIsEnumerable = true;
    }
}

trace("--- Source Array ---");
trace("Length: " + values.length);
trace("Has own index 0: " + values.hasOwnProperty("0"));
trace("Has own index 1: " + values.hasOwnProperty("1"));
trace("Has own index 2: " + values.hasOwnProperty("2"));
trace("Getter index enumerable: " + getterIsEnumerable);
trace("Getter calls before sending: " + getterCalls);

function traceReceivedArray(label, received) {
    trace("--- " + label + " ---");
    trace("Length: " + received.length);

    trace("Has own index 0: " + received.hasOwnProperty("0"));
    trace("Index 0: " + received[0]);

    trace("Has own index 1: " + received.hasOwnProperty("1"));
    trace("Index 1: " + received[1]);

    trace("Has own index 2: " + received.hasOwnProperty("2"));
    trace("Index 2: " + received[2]);
}

var receiver = new LocalConnection();

receiver.onReceiveDirect = function(received) {
    traceReceivedArray("Direct Array Result", received);
};

receiver.onReceiveNested = function(payload) {
    traceReceivedArray("Nested Array Result", payload.values);
};

trace("Connected: " + receiver.connect("sparse_array_getters"));

var sender = new LocalConnection();

// Send the sparse array directly.
sender.send("sparse_array_getters", "onReceiveDirect", values);
trace("Getter calls after direct send: " + getterCalls);

// Send the same sparse array nested inside an ordinary object.
sender.send("sparse_array_getters", "onReceiveNested", {
    values: values
});
trace("Getter calls after nested send: " + getterCalls);
