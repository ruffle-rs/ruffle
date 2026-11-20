var my_xml = new XML("<test><![CDATA[text inside cdata]]>normal text</test>!");
trace(my_xml);

trace("// childNodes.length");
trace(my_xml.childNodes[0].childNodes.length);
trace("// childNodes[0]");
trace(my_xml.childNodes[0].childNodes[0]);
trace("// childNodes[0].nodeType");
trace(my_xml.childNodes[0].childNodes[0].nodeType);
trace("// childNodes[1]");
trace(my_xml.childNodes[0].childNodes[1]);
trace("// childNodes[1].nodeType");
trace(my_xml.childNodes[0].childNodes[1].nodeType);
