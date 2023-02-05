package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			addFrameScript(0, frame1, 1, frame2a, 1, frame2b, 2, frame3, 2, null, 3, frame4);
		}

		function frame1() {
			trace("Frame 1");
		}

		function frame2a() {
			trace("Frame 2a");
		}

		function frame2b() {
			trace("Frame 2b");
		}

		function frame3() {
			trace("Frame 3");
		}

		function frame4() {
			trace("Frame 4");
			stop();
		}
	}
	
}
