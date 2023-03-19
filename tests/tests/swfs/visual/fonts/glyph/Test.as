package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	
	
	public class Test extends MovieClip {
		public var text: TextField;
		
		public function Test() {
			var tf = text.defaultTextFormat;
			tf.size = 200;
			text.setTextFormat(tf);
		}
	}
	
}
