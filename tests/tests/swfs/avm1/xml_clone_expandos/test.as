var src_xml = new XML("<test></test>");
var src_xml_node = src_xml.childNodes[0];

src_xml.expando = "xml_expando";
src_xml_node.expando = "xmlnode_expando";

trace("Shallow copy");
var dst_xml = src_xml.cloneNode(false);
var dst_xml_node = src_xml_node.cloneNode(false);

trace(dst_xml.childNodes.length);
trace(dst_xml_node.childNodes.length);
trace(dst_xml_node.nodeName);

trace("Deep copy");
var deep_xml = src_xml.cloneNode(true);
var deep_xml_node = deep_xml.childNodes[0];

trace(deep_xml.childNodes.length);
trace(deep_xml_node.childNodes.length);
trace(deep_xml_node.nodeName);

trace("All expandos");
trace(src_xml.expando);
trace(src_xml_node.expando);
trace(dst_xml.expando);
trace(dst_xml_node.expando);
trace(deep_xml.expando);
trace(deep_xml_node.expando);

trace("cloneNode w/o arg");
var unk_xml = src_xml.cloneNode();

trace(unk_xml == null);
trace(unk_xml.childNodes.length);
trace(unk_xml.expando);
