package  {
	
	import flash.display.MovieClip;
	
	
	public class SecondFrameChild extends MovieClip {
		
		public static var DUMMY: String = myFunc();
		
		public static function myFunc():String {
			trace("In SecondFrameChild class initializer");
			return "FOO";
		}
		
		public function SecondFrameChild() {
			trace("Constructed SecondFrameChild")
		}
	}
	
}

trace("In SecondFrameChild script initializer");