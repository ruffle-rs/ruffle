package {
	import flash.xml.XMLDocument;
	public class Test {
		public function Test() {
			XML.prettyPrinting = false;
			var data = "<outer><first>  \t</first><second><![CDATA[\t]]></second><third><![CDATA[]]></third></outer>";
			
			XML.ignoreWhitespace = true;
			trace("XML.ignoreWhitespace = true: " + new XML(data));
			
			XML.ignoreWhitespace = false;
			trace("XML.ignoreWhitespace = false: " + new XML(data));
			
			var ignoreWhiteDoc = new XMLDocument();
			ignoreWhiteDoc.ignoreWhite = true;
			ignoreWhiteDoc.parseXML(data);
			trace("XMLDocument ignoreWhite = true: " + ignoreWhiteDoc);
			trace("ignoreWhiteDoc.firstChild.childNodes[1].firstChild.nodeType = " + ignoreWhiteDoc.firstChild.childNodes[1].firstChild.nodeType);
		
			var noIgnoreWhiteDoc = new XMLDocument();
			noIgnoreWhiteDoc.ignoreWhite = false;
			noIgnoreWhiteDoc.parseXML(data);
			trace("XMLDocument ignoreWhite = false: " + noIgnoreWhiteDoc);
			trace("noIgnoreWhiteDoc.firstChild.childNodes[1].firstChild.nodeType = " + noIgnoreWhiteDoc.firstChild.childNodes[1].firstChild.nodeType);	
		}
		
	}
}