package {
	import flash.display.DisplayObjectContainer;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.text.TextField;	
	import flash.display.Shape;
	import flash.display.Stage;
	import flash.display.Sprite;

	public class Test {
		public static function run(container: DisplayObjectContainer) {
			// Based on example from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/BitmapData.html#draw()
			var tf:TextField = new TextField();
			tf.text = "Ruffle bitmap";
			tf.width = 300;
			tf.height = 100;
			
			var myBitmapData:BitmapData = new BitmapData(100,100);
			myBitmapData.draw(tf);
			
			tf.text = "Ruffle TextField";
			tf.y = 30;
			
			var bmp:Bitmap = new Bitmap(myBitmapData, "auto");
			container.addChild(bmp);
			container.addChild(tf);
			
					
			var backgroundRect = new Sprite();
			backgroundRect.graphics.beginFill(0x0088FF);
			backgroundRect.graphics.drawRect(0, 0, 250, 250);
			backgroundRect.x = 90;
			container.addChild(backgroundRect);	
			
			var transparent = new BitmapData(100, 100, true, 0);
			var circle = new Sprite();
			circle.graphics.beginFill(0xff0000);
			circle.graphics.drawCircle(50, 50, 40);
			circle.graphics.endFill();
			
			transparent.draw(circle);
			
			circle.graphics.clear();
			circle.graphics.beginFill(0x00aa00, 0.5);
			circle.graphics.drawCircle(60, 60, 40);
			circle.graphics.endFill();
			
			transparent.draw(circle);
			
			var transparentBmp = new Bitmap(transparent);
			transparentBmp.x = 100;
			container.addChild(transparentBmp);
			
			var opaque = new BitmapData(100, 100, false, 0);
			var circle = new Sprite();
			circle.graphics.beginFill(0xff0000);
			circle.graphics.drawCircle(50, 50, 40);
			circle.graphics.endFill();
			
			opaque.draw(circle);
			
			circle.graphics.clear();
			circle.graphics.beginFill(0x00aa00, 0.5);
			circle.graphics.drawCircle(60, 60, 40);
			circle.graphics.endFill();
			
			opaque.draw(circle);
			
			var opaqueBmp = new Bitmap(opaque);
			opaqueBmp.x = 100;
			opaqueBmp.y = 110;
			container.addChild(opaqueBmp);
		}
	}
}