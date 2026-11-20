XML.prototype.ignoreWhite = true;
var xml = new XML('<root>   <foo> testing </foo>     </root>');

trace("xml.ignoreWhite = true");
trace(xml);
trace("");

trace("root:");
var root = xml.childNodes[0];
trace(" nodeName: " + root.nodeName);
trace(" nodeType: " + root.nodeType);
trace(" childNodes.length: " + root.childNodes.length);

trace("foo:");
var foo = root.childNodes[0];
trace( " nodeName: " + foo.nodeName);
trace( " nodeType: " + foo.nodeType);

trace("text:");
var txt = foo.childNodes[0];
trace( " nodeName: " + txt.nodeName);
trace( " nodeType: " + txt.nodeType);
trace( " nodeValue: "  +txt.nodeValue);
trace("");
trace("");

XML.prototype.ignoreWhite = false;
var xml = new XML('<root>   <foo> testing </foo>     </root>');

trace("xml.ignoreWhite = false");
trace(xml);
trace("");

trace("root:");
var root = xml.childNodes[0];
trace(" nodeName: " + root.nodeName);
trace(" nodeType: " + root.nodeType);
trace(" childNodes.length: " + root.childNodes.length);

trace("white:");
var white = root.childNodes[0];
trace(" nodeName: " + white.nodeName);
trace(" nodeType: " + white.nodeType);
trace(" nodeValue: " + white.nodeValue);


trace("foo:");
var foo = root.childNodes[1];
trace( " nodeName: " + foo.nodeName);
trace( " nodeType: " + foo.nodeType);

trace("text:");
var txt = foo.childNodes[0];
trace( " nodeName: " + txt.nodeName);
trace( " nodeType: " + txt.nodeType);
trace( " nodeValue: "  +txt.nodeValue);
trace("");
trace("");
