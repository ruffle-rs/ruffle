package  {
	
	import flash.display.MovieClip;
	import flash.display.Sprite;
	
	public class MaskTest extends MovieClip {
		public function MaskTest() {
			graphics.beginFill(0xFF00FF);
			graphics.drawCircle(100, 100, 100);
			graphics.endFill();

			var sprite : Sprite = new Sprite();
			sprite.graphics.beginFill(0xFF00FF);
			sprite.graphics.drawRect(20, 0, 160, 200);
			sprite.graphics.endFill();
			
			mask = sprite;
		}
	}	
}
