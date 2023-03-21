package flash.xml
{
   // TODO: Re-implement XMLDocument shim in Rust.
   public class XMLDocument extends XMLNode {

      public var ignoreWhite: Boolean = false;

      public function XMLDocument(input: String = null) {
         super(0, null);
         _isDocument = true;
         parseXML(input);
      }

      public function parseXML(input: String): void {
         // Unlike the new AS3 XML class, XMLDocument allows parsing
         // multiple elements like <a>1</a><b>2</b>. So introduce a wrapper.
         _setXML("<xml>" + input + "</xml>", this.ignoreWhite);
         _children = null;
      }
   }
}

