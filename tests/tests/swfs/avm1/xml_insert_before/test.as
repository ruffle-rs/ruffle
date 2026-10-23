var my_xml = new XML("<test></test><test2></test2>");

var test_xml = my_xml.childNodes[0];
var test2_xml = my_xml.childNodes[1];

trace(test_xml.parentNode === my_xml);
trace(test_xml.previousSibling);
trace(test_xml.nextSibling === test2_xml);
trace(test2_xml.previousSibling === test_xml);
trace(test2_xml.nextSibling);
trace(test2_xml.parentNode === my_xml);

trace("Swapping nodes...");
test2_xml.removeNode();
my_xml.insertBefore(test2_xml, test_xml);

trace(test2_xml.parentNode === my_xml);
trace(test2_xml.previousSibling);
trace(test2_xml.nextSibling === test_xml);
trace(test_xml.previousSibling === test2_xml);
trace(test_xml.nextSibling);
trace(test_xml.parentNode === my_xml);

trace("Failing to swap nodes...");
my_xml.insertBefore(test_xml, test2_xml);

trace(test2_xml.parentNode === my_xml);
trace(test2_xml.previousSibling);
trace(test2_xml.nextSibling === test_xml);
trace(test_xml.previousSibling === test2_xml);
trace(test_xml.nextSibling);
trace(test_xml.parentNode === my_xml);
