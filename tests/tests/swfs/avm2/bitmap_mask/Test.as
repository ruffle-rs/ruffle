package {
	import flash.display.MovieClip;
	import flash.display.Shape;
	import flash.display.BitmapData;
	import flash.geom.Rectangle;
	import flash.display.Bitmap;

	public class Test {
		public function Test(main: MovieClip) {
								var background = new Shape();
					background.graphics.beginFill(0x00FF00);
					background.graphics.drawRect(0, 0, 500, 500);
					background.graphics.endFill();;
					
					main.addChild(background);
			
			for each (var shapeCacheAsBitmap in [false, true]) {
				for each (var bitmapCacheAsBitmap in [false, true]) {
					var shape = new Shape();
					shape.graphics.beginFill(0xaa00bb, 0.5);
					shape.graphics.drawCircle(0, 0, 50);
					shape.graphics.endFill();
					
					var bitmapDataMask = new BitmapData(200, 200, true, 0);
					// FIXME - flash player appears to combine the mask and maskee alpha
					// values in some way (the object is much more transparent than
					// when only one of the masker and maskee has alpha != 1
					// Once we implement this in Ruffle, uncomment this line to test it
					//bitmapDataMask.fillRect(new Rectangle(0, 0, 100, 30), 0x66000100);
					bitmapDataMask.fillRect(new Rectangle(0, 0, 100, 30), 0xFF000100);
					var bitmapMask = new Bitmap(bitmapDataMask);
					
					shape.x = 100;
					shape.y = 100;
					bitmapMask.x = 100;
					bitmapMask.y = 100;
					
					if (shapeCacheAsBitmap) {
						shape.x = 300;
						bitmapMask.x = 300;
					}
					if (bitmapCacheAsBitmap) {
						shape.y = 300;
						bitmapMask.y = 300;
					}
					
					trace("Coords: " + shape.x + " " + shape.y);
					
					main.addChild(bitmapMask);
					shape.mask = bitmapMask;
					trace("Mask: " + shape.mask);
					
					shape.cacheAsBitmap = shapeCacheAsBitmap;
					bitmapMask.cacheAsBitmap = bitmapCacheAsBitmap;
					main.addChild(shape);
				}
			}

		}
	}
}