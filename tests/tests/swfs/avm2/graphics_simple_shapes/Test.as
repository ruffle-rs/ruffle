package  {
	
	import flash.display.MovieClip;
	import flash.display.Graphics;
	import flash.display.Shape;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			simple_shapes();
		}
		
		public function simple_shapes() {
			var child:Shape = new Shape();
            child.graphics.beginFill(0xFF0000);
            child.graphics.drawRect(5, 5, 15, 20);
            child.graphics.lineStyle(2, 0x0000FF);
            child.graphics.drawRoundRect(30, 5, 15, 20, 10);
            child.graphics.lineStyle();
            child.graphics.drawCircle(50, 15, 10);
            child.graphics.lineStyle(4, 0x00FF00);
            child.graphics.drawEllipse(70, 5, 10, 20);
            child.graphics.endFill();
            addChild(child);
		}
	}
	
}
