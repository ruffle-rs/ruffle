package {
	import flash.display.DisplayObjectContainer;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.text.TextField;	
	import flash.display.Shape;
	import flash.display.Stage;

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
		}
	}
}