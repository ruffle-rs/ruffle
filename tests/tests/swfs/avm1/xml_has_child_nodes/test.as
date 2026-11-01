var empty_xml = new XML();

trace(empty_xml.hasChildNodes());

var full_xml = new XML("<test></test>");

trace(full_xml.hasChildNodes());
trace(full_xml.childNodes[0].hasChildNodes());
