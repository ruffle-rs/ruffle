// An enumerable data property at the end of a very deep prototype chain is
// returned by property enumeration, but reading it exceeds AVM1's prototype
// lookup limit. AMF serialization should skip that property and continue.
trace("--- Starting LocalConnection deep prototype serialization test ---");

var payload = new Object();
payload.visible = "retained";

var current = payload;
var prototypeLinks = 255;
for (var i = 0; i < prototypeLinks; i++) {
    var next = new Object();
    current.__proto__ = next;
    current = next;
}
current.tooDeep = "not serialized";

var receiver = new LocalConnection();
receiver.onReceivePayload = function(received) {
    trace("--- Received LocalConnection object ---");
    trace("Received visible property: " + received.visible);
    trace("Received deep property type: " + typeof(received.tooDeep));
    trace("Received own deep property: " + received.hasOwnProperty("tooDeep"));
};
trace("Connected: " + receiver.connect("deep_prototype_get_error"));

var sender = new LocalConnection();
sender.send("deep_prototype_get_error", "onReceivePayload", payload);
