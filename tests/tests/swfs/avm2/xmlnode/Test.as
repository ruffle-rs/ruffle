package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
	}
}


import flash.xml.XMLNode;
import flash.xml.XMLNodeType;

function dumpAttributes(attributes: Object) {
	if (attributes === null) {
		trace("null");
		return;
	}
	if (attributes === undefined) {
		trace("undefined");
		return;
	}

	var keys = [];
	for (var key in keys) {
		keys.push(key);
	}
	keys.sort(); // order would otherwise be arbitrary

	if (keys.length == 0) {
		trace("(empty)");
	} else {
		for each (var key in keys) {
			trace(key + ": " + attributes[key]);
		}
	}
}

function dumpNode(name: String, node: XMLNode) {
	trace("// " + name);
	trace(node);
	trace("");
	
	trace("// " + name + ".attributes");
	dumpAttributes(node.attributes);
	trace("");

	trace("// " + name + ".childNodes");
	trace(node.childNodes);
	trace("");

	trace("// " + name + ".firstChild");
	trace(node.firstChild);
	trace("");

	trace("// " + name + ".lastChild");
	trace(node.lastChild);
	trace("");

	trace("// " + name + ".localName");
	trace(node.localName);
	trace("");

	trace("// " + name + ".namespaceURI");
	trace(node.namespaceURI);
	trace("");

	trace("// " + name + ".nextSibling");
	trace(node.nextSibling);
	trace("");

	trace("// " + name + ".nodeName");
	trace(node.nodeName);
	trace("");

	trace("// " + name + ".nodeType");
	trace(node.nodeType);
	trace("");

	trace("// " + name + ".nodeValue");
	trace(node.nodeValue);
	trace("");

	trace("// " + name + ".parentNode");
	trace(node.parentNode);
	trace("");

	trace("// " + name + ".prefix");
	trace(node.prefix);
	trace("");

	trace("// " + name + ".previousSibling");
	trace(node.previousSibling);
	trace("");

	for (var i = 0; i < node.childNodes.length; i++) {
		dumpNode(name + ".childNodes[" + i + "]", node.childNodes[i]);
	}
}

var unrelatedNode = new XMLNode(XMLNodeType.ELEMENT_NODE, "unrelated");

function makeAndTest(type: String, value: String) {
	var typeid = XMLNodeType[type];

	trace("// var test = new XMLNode(XMLNodeType." + type + ", \"" + value + "\")");
	var test = new XMLNode(typeid, value);

	var list = new XMLNode(XMLNodeType.ELEMENT_NODE, "list");
	var item1 = new XMLNode(XMLNodeType.ELEMENT_NODE, "item");
	item1.attributes = {id: "a", first: true};
	item1.appendChild(new XMLNode(XMLNodeType.TEXT_NODE, "first item! <3"));
	list.appendChild(item1);

	var item2 = new XMLNode(XMLNodeType.ELEMENT_NODE, "item");
	item2.attributes = {id: "b", first: false};
	item2.appendChild(new XMLNode(XMLNodeType.TEXT_NODE, "& me too!"));
	list.appendChild(item2);

	var item4 = new XMLNode(XMLNodeType.ELEMENT_NODE, "item");
	item4.attributes = {id: "d", first: false};
	item4.appendChild(new XMLNode(XMLNodeType.TEXT_NODE, "I should be fourth! :>"));
	list.appendChild(item4);

	var item3 = new XMLNode(XMLNodeType.ELEMENT_NODE, "item");
	item3.attributes = {id: "c", first: false};
	item3.appendChild(new XMLNode(XMLNodeType.TEXT_NODE, "I was \"insertedBefore()\"'d!"));
	list.insertBefore(item3, item4);

	test.appendChild(list);

	var contact = new XMLNode(XMLNodeType.ELEMENT_NODE, "contact:mailbox");
	contact.appendChild(new XMLNode(XMLNodeType.TEXT_NODE, "foo@example.org"));
	test.appendChild(contact);

	dumpNode("test", test);

	trace("");
}

makeAndTest("ELEMENT_NODE", "test");
makeAndTest("TEXT_NODE", "test");
makeAndTest("CDATA_NODE", "test");
makeAndTest("PROCESSING_INSTRUCTION_NODE", "test");
makeAndTest("COMMENT_NODE", "test");
makeAndTest("DOCUMENT_TYPE_NODE", "test");
makeAndTest("XML_DECLARATION", "test");
