package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		
		public function MyChild() {
			trace("Calling MyChild super()");
			super();
			trace("Called MyChild super()");
			this.addFrameScript(0, this.frame1);
		}
	
		private function frame1() {
			stop();
			trace("MyChild frame 1: calling this.parent.gotoAndStop(2)");
			MovieClip(this.parent).gotoAndStop(2);
			trace("MyChild frame 1: called this.parent.gotoAndStop(2)");
		}
	}
	
}
