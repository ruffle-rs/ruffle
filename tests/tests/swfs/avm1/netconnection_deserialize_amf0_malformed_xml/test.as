trace("--- Starting malformed AMF0 XML test ---");

var connection = new NetConnection();

var responder = new Object();
responder.onResult = function(received) {
    trace("--- Malformed AMF0 XML response ---");
    trace("XML type: " + typeof(received));

    if (received != undefined && received != null) {
        trace("XML status: " + received.status);
        trace("XML string: " + received.toString());
    } else {
        trace("XML value: " + received);
    }
};

connection.connect("http://localhost:8000/test1");
connection.call("test.malformedXml", responder);
