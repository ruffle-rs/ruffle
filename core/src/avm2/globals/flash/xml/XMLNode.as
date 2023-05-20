package flash.xml
{
   import __ruffle__.stub_method;
   import __ruffle__.stub_setter;

   // TODO: Re-implement XMLNode shim in Rust.
   public class XMLNode {
      internal var _isDocument: Boolean = false;
      internal var _xml: XML;
      internal var _parent: XMLNode = null;
      internal var _children: Array = null;
      internal var _index: uint = 0;

      public function XMLNode(type: uint, input: String) {
         _setXML(input);
      }

      internal function _setXML(input: String, ignoreWhite: Boolean = false) {
         var previousSettings = XML.AS3::settings();
         try {
            XML.ignoreWhitespace = ignoreWhite;
            _xml = new XML(input);
         } finally {
            XML.AS3::setSettings(previousSettings);
         }
      }

      public function get attributes(): Object {
         var attrs = new Object();
         for each (var attribute in _xml.attributes()) {
            attrs[attribute.localName()] = attribute.toString();
         }
         return attrs;
      }

      public function get childNodes(): Array {
         // Cache the children to avoid creating new XMLNodes on
         // every access.
         if (_children) {
            return _children;
         }

         var children = _xml.children();
         var nodes = new Array();
         for (var i = 0; i < children.length(); i++) {
            var node = new XMLNode(0, children[i].toXMLString());
            node._parent = this;
            node._index = i;
            nodes.push(node);
         }
         _children = nodes;
         return nodes;
      }

      public function get firstChild(): XMLNode {
         return childNodes[0];
      }

      public function get nextSibling(): XMLNode {
         return this._parent.childNodes[this._index + 1];
      }

      public function get nodeName(): String {
         if (_isDocument) {
            return null;
         }
         return _xml.localName();
      }

      public function set nodeName(name: String): void {
         stub_setter("flash.xml.XMLNode", "nodeName");
      }

      public function get nodeType(): uint {
         switch (_xml.nodeKind()) {
            case "attribute":
            default:
               throw new Error("impossible");
            case "comment":
               return XMLNodeType.COMMENT_NODE;
            case "element":
               return XMLNodeType.ELEMENT_NODE;
            case "processing-instruction":
               return XMLNodeType.PROCESSING_INSTRUCTION_NODE;
            case "text":
               return XMLNodeType.TEXT_NODE;
         }
      }

      public function appendChild(node: XMLNode): void {
         stub_method("flash.xml.XMLNode", "appendChild");
      }

      public function toString(): String {
         var string = _xml.toXMLString();
         if (_isDocument) {
            // Hack: Remove <xml> and </xml>
            string = string.substring(5, string.length - 6);
         }
         return string;
      }
   }
}

