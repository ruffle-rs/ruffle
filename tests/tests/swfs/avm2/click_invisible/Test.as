package {
	import flash.display.MovieClip;
	import flash.display.Sprite;
	import flash.display.Shape;
	import flash.geom.Rectangle;
	import flash.events.MouseEvent;
	import flash.display.BitmapData;
	import flash.display.Bitmap;
	import flash.text.TextField;

	public class Test extends MovieClip {
		public function Test() {
			var shapeContainer = new Sprite();
			shapeContainer.name = "MyShapeContainer";
			var shape = new Shape();
			shape.graphics.beginFill(0xFF0000);
			shape.graphics.drawRect(0, 0, 100, 100);
			shape.graphics.endFill();
			shapeContainer.addChild(shape);
			this.addChild(shapeContainer);

			var bitmapContainer = new Sprite();
			bitmapContainer.name = "MyBitmapContainer";
			var bitmapData = new BitmapData(120, 120, false, 0);
			var bitmap = new Bitmap(bitmapData);
			bitmap.x = 150;
			bitmap.name = "MyBitmap";
			bitmapContainer.addChild(bitmap);
			this.addChild(bitmapContainer);

			var text = new TextField();
			text.text = "Some text";
			text.x = 300;
			text.name = "MyText";
			this.addChild(text);

			shape.visible = false;
			bitmap.visible = false;
			text.visible = false;

			this.stage.addEventListener(MouseEvent.CLICK, function(e) {
				trace("Clicked " + e.target.name + " (" + e.stageX + ", " + e.stageY + ")");
			});
		}
	}
}