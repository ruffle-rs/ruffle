package flash.xml
{

import __ruffle__.stub_method;
import flash.xml.XMLNode;
import flash.xml.XMLNodeType;
import flash.xml.XMLNodeType;

   public class XMLDocument extends XMLNode {

      public var ignoreWhite: Boolean = false;

      public function XMLDocument(input: String = null) {
         super(XMLNodeType.ELEMENT_NODE, null);
         if (input != null) {
            parseXML(input);
         }
      }

      public function parseXML(input: String): void {
         // This is something of a hack, but that's somewhat the nature of XMLDocument
         // It accepts things like `<node>...</node> <!-- comment --> foo` which is FOUR children:
         // `<node>...</node>` gets parsed as an element
         // ` ` gets parsed as a text node
         // `<!-- comment -->` gets parsed as a comment
         // ` foo` is another text node
         // To achieve this, just wrap it all together in a parent.

         // TODO: When XML.settings works, provide ignoreWhite to it and disable ignoreComments/ignoreProcessingInstructions

         if (!this.ignoreWhite) {
            stub_method("flash.xml.XMLDocument", "parseXML", "with ignoreWhite = false");
         }
         clear();
         var root = new XML("<xml>" + input + "</xml>");
         for each (var child in root.children()) {
            var node = _convertXmlNode(child);
            if (node != null) {
               appendChild(node);
            }
         }
      }

      private function _convertXmlNode(original: XML): XMLNode {
         var nodeType = _convertXmlNodeType(original.nodeKind());
         if (nodeType == 0) {
            return null;
         }
         var nodeValue = nodeType == XMLNodeType.ELEMENT_NODE ? original.name() : original.toString();
         var result = new XMLNode(nodeType, nodeValue);
         for each (var originalChild in original.children()) {
            var child = _convertXmlNode(originalChild);
            if (child != null) {
               result.appendChild(child);
            }
         }
         var attributeList = original.attributes();
         var attributes = {};
         for each (var attribute in attributeList) {
            attributes[attribute.name()] = attribute.toString();
         }
         result.attributes = attributes;
         return result;
      }

      private function _convertXmlNodeType(kind: String): uint {
         if (kind == "text") {
            return XMLNodeType.TEXT_NODE;
         }
         if (kind == "comment") {
            return XMLNodeType.COMMENT_NODE;
         }
         if (kind == "processing-instruction") {
            return XMLNodeType.PROCESSING_INSTRUCTION_NODE;
         }
         if (kind == "element") {
            return XMLNodeType.ELEMENT_NODE;
         }
         return 0;
      }

      public function createElement(name:String): XMLNode {
         return new XMLNode(XMLNodeType.ELEMENT_NODE, name);
      }
   }
}

