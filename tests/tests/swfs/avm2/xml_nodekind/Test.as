package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			// Taken from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/XML.html#nodeKind()
			// Modified to run with what ruffle support
			//XML.ignoreComments = false;
			
			var xml:XML = 
				<example id="10">
					<![CDATA[some cdata]]>
					and some text
				</example>;
			
			trace(xml.nodeKind()); // element
			trace(xml.children()[0].nodeKind()); // text
			trace(xml.children()[1].nodeKind()); // text
		}
	}
	
}
