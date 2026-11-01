var my_xml = new XML("<test1></test1><test2></test2><test3></test3><test4></test4>");

var t1_xml = my_xml.childNodes[0];
var t2_xml = my_xml.childNodes[1];
var t3_xml = my_xml.childNodes[2];
var t4_xml = my_xml.childNodes[3];

trace(t1_xml.previousSibling);
trace(t1_xml.nextSibling === t2_xml);
trace(t2_xml.previousSibling === t1_xml);
trace(t2_xml.nextSibling === t3_xml);
trace(t3_xml.previousSibling === t2_xml);
trace(t3_xml.nextSibling === t4_xml);
trace(t4_xml.previousSibling === t3_xml);
trace(t4_xml.nextSibling);

trace(my_xml.previousSibling);
trace(my_xml.nextSibling);
