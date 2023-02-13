package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		public function Test() {
			this.stage.addChild(new LazySprite());
			trace("Success!");
		}
	}
	
}
