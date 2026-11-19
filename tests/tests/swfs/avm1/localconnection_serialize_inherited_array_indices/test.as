var hadOwnArrayPrototypeZero = Array.prototype.hasOwnProperty("0");
var previousArrayPrototypeZero = Array.prototype[0];

// Make index 0 enumerable through the array's prototype, not its own properties.
Array.prototype[0] = "inherited_zero";

var values = new Array();
values[1] = "own_one";

trace("--- Source Array ---");
trace("Length: " + values.length);
trace("Has own index 0: " + values.hasOwnProperty("0"));
trace("Index 0: " + values[0]);
trace("Has own index 1: " + values.hasOwnProperty("1"));
trace("Index 1: " + values[1]);

// Nest the array in an ordinary object to exercise the same
// serialize_value_to_writer path as the function-valued-elements test.
var payload = new Object();
payload.values = values;

var receiver = new LocalConnection();
receiver.onReceiveArray = function(received) {
    // Restore the prototype before inspecting the received array.
    // This prevents the inherited property from masking whether
    // index 0 was actually serialized.
    if (hadOwnArrayPrototypeZero) {
        Array.prototype[0] = previousArrayPrototypeZero;
    } else {
        delete Array.prototype[0];
    }

    var receivedValues = received.values;

    trace("--- Received Array ---");
    trace("Length: " + receivedValues.length);

    trace("Has own index 0: " + receivedValues.hasOwnProperty("0"));
    trace("Index 0: " + receivedValues[0]);

    trace("Has own index 1: " + receivedValues.hasOwnProperty("1"));
    trace("Index 1: " + receivedValues[1]);
};
receiver.connect("inherited_array_indices");

var sender = new LocalConnection();
sender.send("inherited_array_indices", "onReceiveArray", payload);
