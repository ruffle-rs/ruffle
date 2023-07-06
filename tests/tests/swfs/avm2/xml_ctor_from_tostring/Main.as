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
		
			var xmlObj = <outer/>;
			var xmlCopy = new XML(xmlObj);
			var xmlCast = XML(xmlObj);
			trace("xmlCopy().toXMLString(): " + xmlCopy.toXMLString());
			trace("xmlObj === xmlCopy: " + (xmlObj === xmlCopy));
			trace("xmlObj === xmlCast: " + (xmlObj === xmlCast));
		
		
			var listFromSingle = XMLList(xmlObj);
			trace("listFromSingle[0] === xmlObj: " + (listFromSingle[0] === xmlObj));
			var newListFromSingle = new XMLList(xmlObj);
			trace("newListFromSingle[0] === xmlObj: " + (newListFromSingle[0] === xmlObj));
			trace("new XMLList(listFromSingle) === listFromSingle: " + (new XMLList(listFromSingle) === listFromSingle));
		
			var emptyList = new XMLList();
			trace("emptyList.toString(): " + emptyList.toString());
			trace("emptyList.toXMLString(): " + emptyList.toXMLString());
		
			try {
				new XML(emptyList);
			} catch (e) {
				trace("Caught error: " + e);
				trace(e.errorID);
			}
		
			var singleList = new XMLList("<outer><inner>Hello</inner><second>World</second></outer>");
			var xmlFromSingle = XML(singleList);
			trace("xmlFromSingle === singleList[0]: " + (xmlFromSingle === singleList[0]));
			var newXMLFromSingle = new XML(singleList);
			trace("newXMLFromSingle === singleList[0]: " + (newXMLFromSingle === singleList[0]));

		
			var multiList = new XMLList("<first>Hello</first><second>World</second>");
		
			var castCopy = XMLList(multiList);
			var ctorCopy = new XMLList(multiList);
		
			trace("castCopy equal: " + (multiList === castCopy));
			trace("ctorCopy equal: " + (multiList === ctorCopy));
			
			try {
				new XML(multiList);
			} catch (e) {
				trace("Caught error: " + e);
				trace(e.errorID);
			}
		
			try {
				trace(new XML("<Hello<"));
			} catch (e) {
				trace("Caught parsing error: " + e);
				trace(e.errorID);
			}
		}
	}
	
}
