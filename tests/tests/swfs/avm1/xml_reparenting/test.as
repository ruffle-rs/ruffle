trace("// reparenting in self");
var xml = new XML("<a><b/></a><c/>");
trace(xml);
xml.appendChild(xml.firstChild.firstChild);
trace("// after");
trace(xml);

trace("// reparenting in other");
xml = new XML("<a><b/></a>");
var other = new XML("<c/>");
trace(xml);
trace(other);
other.insertBefore(xml.firstChild.firstChild, other.firstChild);
trace("// after");
trace(xml);
trace(other);

trace("// can't reparent in descendent");
xml = new XML("<a><b/></a>");
trace(xml);
xml.firstChild.firstChild.appendChild(xml.firstChild);
trace("// after");
trace(xml);
