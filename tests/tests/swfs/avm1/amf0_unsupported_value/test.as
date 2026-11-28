// AMF0 marker 0x0D is the Unsupported value marker. It should deserialize
// to ActionScript undefined rather than failing the NetConnection response.
trace("--- AMF0 Unsupported value deserialization ---");

var connection = new NetConnection();

var responder = new Object();
responder.onResult = function(value) {
    trace("Result type: " + typeof(value));
    trace("Result value: " + value);
};
responder.onStatus = function(info) {
    trace("Responder status: " + info.code);
};

connection.onStatus = function(info) {
    trace("Connection status: " + info.code);
};

connection.connect("http://localhost:8000/test1");
connection.call("test.unsupported", responder);
