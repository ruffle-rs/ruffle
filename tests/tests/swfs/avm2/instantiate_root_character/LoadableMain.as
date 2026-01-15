package  {
	
	import flash.display.MovieClip;
	import flash.display.Sprite;
	
	
	public class LoadableMain extends MovieClip {
		
		public var myChild:Sprite;
		
		public function LoadableMain(arg:String = null) {
			trace("Instantiated LoadableMain with arg: " + arg);
		}
	}
	
}
