// Test AMF0 XML deserialization with the global XML constructor available.

var receiver = new LocalConnection();

receiver.onReceiveData = function(payload) {
    var items = payload.items;

    var xml0 = items[0];
    var xml1 = items[1];

    trace("XML value 0 type: " + typeof(xml0));
    trace("XML value 1 type: " + typeof(xml1));

    // Both received values should be XML instances.
    trace("XML 0 instanceof XML: " + (xml0 instanceof XML));
    trace("XML 1 instanceof XML: " + (xml1 instanceof XML));

    // Independently deserialized XML values should use the same class.
    trace("XMLs have same constructor: " +
        (xml0.constructor === xml1.constructor));

    // Check the first XML document.
    var root0 = xml0.firstChild;

    if (root0 == undefined || root0 == null) {
        trace("XML 0 has no firstChild");
    } else {
        trace("XML 0 root: " + root0.nodeName);
        trace("XML 0 attribute: " + root0.attributes.id);

        var child0 = root0.firstChild;

        if (child0 == undefined || child0 == null) {
            trace("XML 0 has no child");
        } else {
            trace("XML 0 child: " + child0.nodeName);

            var text0 = child0.firstChild;

            if (text0 == undefined || text0 == null) {
                trace("XML 0 has no text node");
            } else {
                trace("XML 0 text: " + text0.nodeValue);
            }
        }
    }

    // Check the second XML document.
    var root1 = xml1.firstChild;

    if (root1 == undefined || root1 == null) {
        trace("XML 1 has no firstChild");
    } else {
        trace("XML 1 root: " + root1.nodeName);
        trace("XML 1 attribute: " + root1.attributes.id);

        var child1 = root1.firstChild;

        if (child1 == undefined || child1 == null) {
            trace("XML 1 has no child");
        } else {
            trace("XML 1 child: " + child1.nodeName);

            var text1 = child1.firstChild;

            if (text1 == undefined || text1 == null) {
                trace("XML 1 has no text node");
            } else {
                trace("XML 1 text: " + text1.nodeValue);
            }
        }
    }
};

trace("Connected: " + receiver.connect("xml_available"));

// Construct two separate XML documents.
var xml0 = new XML(
    "<first id='one'><child>alpha</child></first>"
);

var xml1 = new XML(
    "<second id='two'><child>beta</child></second>"
);

var payload = {
    items: [xml0, xml1]
};

var sender = new LocalConnection();
sender.send("xml_available", "onReceiveData", payload);
