package  {
	
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			XML.prettyPrinting = false;
			
			var byteArray: ByteArray = new ByteArray();
			byteArray.writeUTFBytes("<foo><bar>test</bar></foo>");
			byteArray.position = 0;
			
			trace("// new XML(byteArray).bar");
			trace(new XML(byteArray).bar);
			trace("");
			
			var objWithToString = {};
			objWithToString.toString = function() { return "<foo><bar>test</bar></foo>"; };
			trace("// new XML(objWithToString).bar");
			trace(new XML(objWithToString).bar);
		
			var xmlObj = <outer></outer>;
			var xmlCopy = new XML(xmlObj);
			trace("xmlCopy().toXMLString(): " + xmlCopy.toXMLString());
			trace("xmlObj === xmlCopy: " + (xmlObj === xmlCopy));
		
			var emptyList = new XMLList();
			try {
				new XML(emptyList);
			} catch (e) {
				trace("Caught error: " + e);
				trace(e.errorID);
			}
		
			var singleList = new XMLList("<outer><inner>Hello</inner><second>World</second></outer>");
			trace("new XML(singleList): " + new XML(singleList));
		
			var multiList = new XMLList("<first>Hello</first><second>World</second>");
			try {
				new XML(multiList);
			} catch (e) {
				trace("Caught error: " + e);
				trace(e.errorID);
			}
		}
	}
	
}
