package flash.xml
{

import flash.xml.XMLNode;
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

         var oldSettings = XML.AS3::settings();
         var newSettings = XML.AS3::defaultSettings();
         newSettings.ignoreWhitespace = this.ignoreWhite;
         XML.AS3::setSettings(newSettings);

         try {
            clear();
            var root = new XML("<xml>" + input + "</xml>");
            for each (var child in root.children()) {
               appendChild(_convertXmlNode(child));
            }
         } finally {
            XML.AS3::setSettings(oldSettings);
         }
      }

      private function _convertXmlNode(original: XML): XMLNode {
         var nodeType = _convertXmlNodeType(original.nodeKind());
         var nodeValue = nodeType == XMLNodeType.ELEMENT_NODE ?
            _convertXmlName(original) : original.toString();
         var result = new XMLNode(nodeType, nodeValue);
         for each (var originalChild in original.children()) {
            result.appendChild(_convertXmlNode(originalChild));
         }

         var attributes = {};
         for each (var attribute in original.attributes()) {
            attributes[_convertXmlName(attribute)] = attribute.toString();
         }
         for each (var ns in original.namespaceDeclarations()) {
            var name = "xmlns";
            if (ns.prefix) {
               name += ":" + ns.prefix;
            }
            attributes[name] = ns.uri;
         }

         result.attributes = attributes;
         return result;
      }

      private function _convertXmlName(node: XML): String {
         var ns = node.namespace();
         if (ns.prefix) {
            return ns.prefix + ":" + node.localName();
         }
         return node.localName();
      }

      private function _convertXmlNodeType(kind: String): uint {
         if (kind == "text") {
            return XMLNodeType.TEXT_NODE;
         }
         if (kind == "comment") {
            return XMLNodeType.COMMENT_NODE;
         }
         if (kind == "element") {
            return XMLNodeType.ELEMENT_NODE;
         }
         throw new Error("Invalid XML Node kind '" + kind + "' found whilst constructing (legacy) XMLDocument");
      }

      public function createElement(name:String): XMLNode {
         return new XMLNode(XMLNodeType.ELEMENT_NODE, name);
      }

      public function createTextNode(text:String): XMLNode {
         return new XMLNode(XMLNodeType.TEXT_NODE, text);
      }
   }
}

