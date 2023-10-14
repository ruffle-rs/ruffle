package  {
	
	import flash.display.MovieClip;
	
	
	public class FourthFrameChild extends MovieClip {
		
		public static var DUMMY: String = myFunc();
		
		public static function myFunc():String {
			trace("In FourthFrameChild class initializer");
			return "FOO";
		}
		
		public function SecondFrameChild() {
			trace("Constructed FourthFrameChild")
		}
	}
	
}

trace("In FourthFrameChild script initializer");