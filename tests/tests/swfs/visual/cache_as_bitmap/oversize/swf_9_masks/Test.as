package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			test.width = 2879;
			test.height = 500;
			test.x = (stage.stageWidth / 2) - (test.width / 2);
			test.y = (stage.stageHeight / 2) - (test.height / 2);
			
			bg.mask = test;
		}
	}
	
}
