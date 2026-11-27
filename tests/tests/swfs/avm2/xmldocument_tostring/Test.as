package {
    import flash.display.Sprite;
    import flash.xml.XMLDocument;
    import flash.xml.XMLNode;
    import flash.xml.XMLNodeType;

    public class Test extends Sprite {
        public function Test() {
            var doc:XMLDocument = new XMLDocument("<root><child>hello</child></root>");
            trace("default: " + doc.toString());

            doc.xmlDecl = '<?xml version="1.0"?>';
            trace("with xmlDecl: " + doc.toString());

            doc.docTypeDecl = '<!DOCTYPE root SYSTEM "root.dtd">';
            trace("with xmlDecl and docTypeDecl: " + doc.toString());

            doc.xmlDecl = null;
            trace("with docTypeDecl only: " + doc.toString());

            // Empty document cases
            var emptyDoc:XMLDocument = new XMLDocument();
            trace("empty default: [" + emptyDoc.toString() + "]");

            emptyDoc.xmlDecl = '<?xml version="1.0"?>';
            trace("empty with xmlDecl: [" + emptyDoc.toString() + "]");

            emptyDoc.docTypeDecl = '<!DOCTYPE root SYSTEM "root.dtd">';
            trace("empty with both: [" + emptyDoc.toString() + "]");

            // Non-null nodeName cases
            emptyDoc.nodeName = "custom";
            trace("empty with nodeName and decls: [" + emptyDoc.toString() + "]");

            doc.nodeName = "custom";
            trace("non-empty with nodeName and docTypeDecl: " + doc.toString());

            // 1. Multiple children
            var multiDoc:XMLDocument = new XMLDocument("<a>1</a><b>2</b>text<c>3</c>");
            trace("multiple children: " + multiDoc.toString());

            // 2. Manipulating firstChild, lastChild, nextSibling, parentNode, and childNodes
            var linkDoc:XMLDocument = new XMLDocument("<one/><two/><three/>");
            var first:XMLNode = linkDoc.firstChild;
            var second:XMLNode = first.nextSibling;
            var third:XMLNode = second.nextSibling;

            second.parentNode = null;
            trace("null parentNode on middle child: " + linkDoc.toString());

            linkDoc.lastChild = first;
            trace("modified lastChild: " + linkDoc.toString());

            first.nextSibling = third;
            trace("modified nextSibling: " + linkDoc.toString());

            linkDoc.firstChild = null;
            trace("null firstChild: " + linkDoc.toString());

            var arrDoc:XMLDocument = new XMLDocument("<x/><y/>");
            arrDoc.childNodes.reverse();
            trace("reversed childNodes: " + arrDoc.toString());

            // 3. Subclass overriding hasChildNodes()
            var subEmpty:SubXMLDocument = new SubXMLDocument();
            subEmpty.logCalls = true;
            trace("subEmpty.toString(): " + subEmpty.toString());

            var subFull:SubXMLDocument = new SubXMLDocument("<item/>");
            subFull.logCalls = true;
            trace("subFull.toString(): " + subFull.toString());

            subFull.forceResult = true;
            subFull.forcedValue = false;
            trace("subFull forced hasChildNodes=false: " + subFull.toString());

            subEmpty.forceResult = true;
            subEmpty.forcedValue = true;
            trace("subEmpty forced hasChildNodes=true: [" + subEmpty.toString() + "]");
            
            // 4. Child XMLNode overriding toString(), including mutating nextSibling during iteration
	    var overrideDoc:XMLDocument = new XMLDocument();
	    var c1:CustomXMLNode = new CustomXMLNode(XMLNodeType.ELEMENT_NODE, "c1");
	    var c2:CustomXMLNode = new CustomXMLNode(XMLNodeType.ELEMENT_NODE, "c2");
	    var c3:CustomXMLNode = new CustomXMLNode(XMLNodeType.ELEMENT_NODE, "c3");
	    overrideDoc.appendChild(c1);
	    overrideDoc.appendChild(c2);
	    overrideDoc.appendChild(c3);
	    trace("child toString override: " + overrideDoc.toString());

	    c1.newNextSibling = c3;
	    trace("child toString mutates nextSibling to c3: " + overrideDoc.toString());

	    c1.newNextSibling = null;
	    c1.clearNextSibling = true;
	    trace("child toString mutates nextSibling to null: " + overrideDoc.toString());
        }
    }
}

import flash.xml.XMLDocument;
import flash.xml.XMLNode;
class SubXMLDocument extends XMLDocument {
    public var logCalls:Boolean = false;
    public var forceResult:Boolean = false;
    public var forcedValue:Boolean = false;

    public function SubXMLDocument(input:String = null) {
        super(input);
    }
    
    override public function hasChildNodes():Boolean {
        if (logCalls) {
            trace("SubXMLDocument.hasChildNodes() called");
        }
        if (forceResult) {
            return forcedValue;
        }
        return super.hasChildNodes();
    }
}

class CustomXMLNode extends XMLNode {
    public var newNextSibling:XMLNode = null;
    public var clearNextSibling:Boolean = false;

    public function CustomXMLNode(type:uint, input:String) {
        super(type, input);
    }

    override public function toString():String {
        trace("CustomXMLNode.toString() called for " + nodeName);
        if (clearNextSibling) {
            this.nextSibling = null;
        } else if (newNextSibling != null) {
            this.nextSibling = newNextSibling;
        }
        return "[" + nodeName + "]";
    }
}
