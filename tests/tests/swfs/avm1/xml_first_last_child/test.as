var my_xml = new XML("<test></test><test2></test2><test3>text</test3>");

var test1_xml = my_xml.childNodes[0];
var test2_xml = my_xml.childNodes[1];
var test3_xml = my_xml.childNodes[2];
var test3_xml_text = test3_xml.childNodes[0];

trace(my_xml.firstChild === test1_xml);
trace(my_xml.lastChild === test3_xml);
trace(test1_xml.firstChild);
trace(test1_xml.lastChild);
trace(test3_xml.firstChild === test3_xml.lastChild);
trace(test3_xml.firstChild === test3_xml_text);
trace(teste_xml_text.firstChild);
trace(teste_xml_text.lastChild);
