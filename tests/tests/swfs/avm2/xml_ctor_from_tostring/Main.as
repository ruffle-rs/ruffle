package  {
	
	import flash.display.MovieClip;
	import flash.utils.ByteArray;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
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
		}
	}
	
}
