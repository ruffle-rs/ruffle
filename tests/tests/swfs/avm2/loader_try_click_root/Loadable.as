package  {
	
	import flash.display.MovieClip;
	
	
	public class Loadable extends MovieClip {
		
		public var mouseDisabled:MovieClip;
		
		
		public function Loadable() {
			this.mouseDisabled.mouseEnabled = false;
		}
	}
	
}
