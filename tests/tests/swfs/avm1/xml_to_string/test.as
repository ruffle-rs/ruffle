var my_xml = new XML("<test></test>\nI'm a <b>text node</b>!\n<продукция>Название_Name</продукция>");
trace(my_xml);

var test_xml = my_xml.childNodes[0];
var test2_xml = new XMLNode(1, "test2");
test_xml.appendChild(test2_xml);
trace(my_xml);

var test2_text = new XMLNode(3, "\"Test\" Node 2");
test2_xml.appendChild(test2_text);
trace(my_xml);

my_xml.nodeName = "newNodeName";
my_xml.nodeName = undefined; // undefined is ignored.
trace(my_xml);
trace(my_xml.childNodes[4]);
