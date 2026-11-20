var my_xml = new XML("<test><test2></test2></test><test3></test3>");

var test_xml = my_xml.childNodes[0];
var test2_xml = my_xml.childNodes[0].childNodes[0];
var test3_xml = my_xml.childNodes[1];

trace(test_xml.parentNode === my_xml);
trace(test_xml.previousSibling);
trace(test_xml.nextSibling === test3_xml);
trace(test2_xml.parentNode === test_xml);
trace(test2_xml.previousSibling);
trace(test2_xml.nextSibling);
trace(test3_xml.parentNode === my_xml);
trace(test3_xml.previousSibling === test_xml);
trace(test3_xml.nextSibling);

trace("After node removal");
test_xml.removeNode();

trace(my_xml.childNodes[0] === test3_xml);
trace(my_xml.childNodes.length);
trace(test3_xml.parentNode === my_xml);
trace(test3_xml.previousSibling);
trace(test3_xml.nextSibling);
trace(test_xml.parentNode);
trace(test_xml.previousSibling);
trace(test_xml.nextSibling);
trace(test_xml.childNodes[0] === test2_xml);
trace(test2_xml.parentNode === test_xml);
trace(test2_xml.previousSibling);
trace(test2_xml.nextSibling);
