// Test AMF0 XML deserialization when the global XML constructor
// has been removed.

var OriginalXML = XML;

var receiver = new LocalConnection();

receiver.onReceiveData = function(payload) {
    var items = payload.items;

    var xml0 = items[0];
    var xml1 = items[1];

    trace("XML value 0 type: " + typeof(xml0));
    trace("XML value 1 type: " + typeof(xml1));

    // Compare against the original XML constructor.
    trace("XML 0 instanceof OriginalXML: " +
        (xml0 instanceof OriginalXML));
    trace("XML 1 instanceof OriginalXML: " +
        (xml1 instanceof OriginalXML));

    // Determine whether the two values share a constructor.
    trace("XMLs have same constructor: " +
        (xml0.constructor === xml1.constructor));

    // Investigate the constructors actually returned.
    trace("XML 0 constructor type: " + typeof(xml0.constructor));
    trace("XML 0 constructor is Object: " +
        (xml0.constructor === Object));
    trace("XML 0 constructor is XMLNode: " +
        (xml0.constructor === XMLNode));

    trace("XML 1 constructor is Object: " +
        (xml1.constructor === Object));
    trace("XML 1 constructor is XMLNode: " +
        (xml1.constructor === XMLNode));

    // Inspect the returned values.
    trace("XML 0 toString: " + xml0.toString());
    trace("XML 1 toString: " + xml1.toString());

    trace("XML 0 nodeName: " + xml0.nodeName);
    trace("XML 0 nodeValue: " + xml0.nodeValue);

    trace("XML 1 nodeName: " + xml1.nodeName);
    trace("XML 1 nodeValue: " + xml1.nodeValue);

    // Inspect the document roots, if present.
    var root0 = xml0.firstChild;
    var root1 = xml1.firstChild;

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

    // Check ordinary object deserialization and reference identity.
    trace("Object property: " + items[2].marker);
    trace("Shared object identity: " + (items[2] === items[3]));
    trace("Shared second type: " + typeof(items[3]));
    trace("Shared second marker: " + items[3].marker);
};

trace("Connected: " + receiver.connect("xml_unavailable"));

// Construct two separate XML documents.
var xml0 = new XML(
    "<first id='one'><child>alpha</child></first>"
);

var xml1 = new XML(
    "<second id='two'><child>beta</child></second>"
);

var shared = {marker: "shared"};

var payload = {
    items: [xml0, xml1, shared, shared]
};

// Remove the global XML constructor before deserialization.
_global.XML = undefined;

var sender = new LocalConnection();
sender.send("xml_unavailable", "onReceiveData", payload);
