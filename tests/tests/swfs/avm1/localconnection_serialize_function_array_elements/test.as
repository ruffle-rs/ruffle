function arrayElementFunction() {
    return "function_result";
}

var values = new Array();
values[0] = arrayElementFunction;
values[1] = "keep";

trace("--- Source Array ---");
trace("Length: " + values.length);
trace("Index 0 type: " + typeof values[0]);
trace("Index 1: " + values[1]);

function traceReceivedArray(label, receivedValues) {
    trace("--- " + label + " ---");
    trace("Length: " + receivedValues.length);

    trace("Has own index 0: " + receivedValues.hasOwnProperty("0"));
    trace("Index 0 type: " + typeof receivedValues[0]);
    trace("Index 0: " + receivedValues[0]);

    trace("Has own index 1: " + receivedValues.hasOwnProperty("1"));
    trace("Index 1 type: " + typeof receivedValues[1]);
    trace("Index 1: " + receivedValues[1]);
}

var receiver = new LocalConnection();
receiver.onReceiveDirect = function(receivedValues) {
    traceReceivedArray("Direct Array Result", receivedValues);
};
receiver.onReceiveNested = function(received) {
    traceReceivedArray("Nested Array Result", received.values);
};
receiver.connect("function_array_elements");

var sender = new LocalConnection();

// Send the array directly to exercise serialize_array's ECMA-array path.
sender.send("function_array_elements", "onReceiveDirect", values);

// Also keep the nested case, which exercises serialize_value_to_writer.
var payload = new Object();
payload.values = values;
sender.send("function_array_elements", "onReceiveNested", payload);
