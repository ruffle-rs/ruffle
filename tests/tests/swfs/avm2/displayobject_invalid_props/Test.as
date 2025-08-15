package  {
	
	import flash.display.MovieClip;
	import flash.display.Shape;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var shape = new Shape();
			shape.graphics.beginFill(0xFF0000);
			shape.graphics.drawRect(0, 0, 100, 100);
			shape.x = 10;
			shape.y = 10;
			shape.scaleX = 2.0;
			shape.scaleY = 3.5;
			shape.rotation = NaN;
			trace("Rotation: " + shape.rotation);
			trace("Shape matrix before: " + shape.transform.matrix);
			shape.scaleX = 1.5;
			shape.scaleY = 4.5;
			trace("Shape matrix after: " + shape.transform.matrix);
			this.addChild(shape);
		}
	}
	
}
