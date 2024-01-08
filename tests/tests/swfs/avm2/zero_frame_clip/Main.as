package  {
	
	import flash.display.MovieClip;

	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			var child = MovieClip(this.getChildAt(0));
			trace("Child: " + child + " totalFrames: " + child.totalFrames);
			child.addChild(new NormalChild());
		}
	}
	
}
