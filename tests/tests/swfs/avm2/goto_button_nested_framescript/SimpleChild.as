package  {
	
	import flash.display.MovieClip;
	
	
	public class SimpleChild extends MovieClip {
		
		
		public function SimpleChild() {
			trace("Calling SimpleChild super()");
			super();
			trace("Called SimpleChild super()");
		}
	}
	
}
