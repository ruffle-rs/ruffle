function trace_xmlnode(node:XMLNode) {
    trace("Type: " + node.nodeType);
    trace("Name: " + node.nodeName);
    trace("Value: " + node.nodeValue);

    if (node.attributes) {
        trace("Attributes defined");
        for (var key in node.attributes) {
            trace(key + ": " + node.attributes[key]);
        }
    } else {
        trace("Attributes not defined");
    }

    if (node.childNodes) {
        trace("Has " + node.childNodes.length + " children");
        for (var i = 0; i < node.childNodes.length; i += 1) {
            trace_xmlnode(node.childNodes[i]);
        }
    } else {
        trace("Children not defined");
    }
}

var my_xml = new XML("");
trace("Empty XML document");
trace_xmlnode(my_xml);

my_xml.parseXML("<test>I'm a test text node.</test>Here's more text.");
trace("Replaced XML document");
trace_xmlnode(my_xml);

var test_xml = my_xml.childNodes[0];
var test_text_xml = my_xml.childNodes[1];

my_xml.parseXML("Now it's different: <test2></test2>");

trace("Second replaced XML document");
trace_xmlnode(my_xml);
trace("First document's first node");
trace_xmlnode(test_xml);
trace("Parent: " + test_xml.parentNode);
trace("First document's second node");
trace_xmlnode(test_text_xml);
trace("Parent: " + test_text_xml.parentNode);
