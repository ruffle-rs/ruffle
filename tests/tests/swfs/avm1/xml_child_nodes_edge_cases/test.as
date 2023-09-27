var my_xml = new XML("<lst><a></a><b></b></lst>");
var a = my_xml.firstChild.childNodes;
trace(a);
my_xml.firstChild.appendChild(my_xml.createElement("c"))
trace(a);
a.push(my_xml.createElement("d"))
trace(a);
my_xml.firstChild.appendChild(my_xml.createElement("e"))
trace(a);