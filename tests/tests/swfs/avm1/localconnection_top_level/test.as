// Create a TextField (DisplayObject)
_root.createTextField("test_tf", 999, 0, 0, 100, 20);
var topDisplayObject = _root.test_tf;

var topDate = new Date(1672531200000); // Fixed UTC Timestamp (Jan 1, 2023)
var topXML = new XML("<root><node attr='test'>AVM1</node></root>");

var topFunction = function(value) {
    return "Function called: " + value;
};

var lcReceiver = new LocalConnection();
lcReceiver.onReceiveData = function(date, xml, dispObj, func, n) {
    trace("--- LocalConnection Deserialization Results ---");
    trace("Date: " + date.getTime());
    trace("XML: " + xml.firstChild.nodeName);
    trace("DisplayObject: " + dispObj);
    trace("Function: " + func);

    if (func != undefined) {
        trace("Function Result: " + func("test"));
    }
};
lcReceiver.connect("amf0_test_connection");

trace("--- Testing LocalConnection ---");
var lcSender = new LocalConnection();
lcSender.send(
    "amf0_test_connection", 
    "onReceiveData", 
    topDate, 
    topXML, 
    topDisplayObject,
	topFunction
);
