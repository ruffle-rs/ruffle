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
var test_xml = my_xml.createElement("test");
var test_xml_text = my_xml.createTextNode("This is a test");
test_xml.appendChild(test_xml_text);
my_xml.appendChild(test_xml);

trace_xmlnode(my_xml);
