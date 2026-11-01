var my_xml = new XML("<test1><test2><test3><test4></test4></test3></test2></test1>");

var test1_xml = my_xml.childNodes[0];
var test2_xml = test1_xml.childNodes[0];
var test3_xml = test2_xml.childNodes[0];
var test4_xml = test3_xml.childNodes[0];

trace(test4_xml.parentNode === test3_xml);
trace(test3_xml.parentNode === test2_xml);
trace(test2_xml.parentNode === test1_xml);
trace(test1_xml.parentNode === my_xml);
trace(my_xml.parentNode);
