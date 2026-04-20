package {
	import flash.display.Sprite;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.filters.BevelFilter;
	import flash.filters.BitmapFilter;
	import flash.filters.BlurFilter;
	import flash.filters.ConvolutionFilter;
	import flash.filters.DropShadowFilter;
	import flash.filters.GlowFilter;
	import flash.geom.Point;
	import flash.geom.Rectangle;

	[SWF(width="600", height="500")]
	public class Test extends Sprite {
		private static var BG_CELL_SIZE:uint = 9;
		private static var IMAGE_SIZE:uint = 100;

		public function Test():void {
			var bg:BitmapData = createCheckeredBackground(
				stage.stageWidth,
				stage.stageHeight
			);
			addChild(new Bitmap(bg));

			var src:BitmapData = createSource();
			var count:uint = 0;

			addImage(count++, src);
			addImage(count++, testFilter(src, new BlurFilter(), new Point(5, 5)));
			addImage(count++, testFilter(src, new BlurFilter(), new Point(-10, -10)));
			addImage(count++, testFilter(src, new BlurFilter(8, 8, 2), new Point(-20, -5)));
			addImage(count++, testFilter(src, new GlowFilter(), new Point(-15, -15)));
			addImage(count++, testFilter(src, new DropShadowFilter(), new Point(-10, -10)));
			addImage(count++, testFilter(src, new BevelFilter(), new Point(-10, -10)));
			addImage(count++, testFilter(src, new ConvolutionFilter(3, 3, [0, -1, 0, -1, 5, -1, 0, -1, 0]), new Point(-10, -10)));
			addImage(count++, testFilter(src, new BlurFilter(), new Point(-50, -50)));
			addImage(count++, testFilter(src, new BlurFilter(), new Point(-5, 10)));
			addImage(count++, testFilter(src, new BlurFilter(), new Point(10, -5)));
			addImage(count++, testFilter(src, new BlurFilter(), new Point(-99, -99)));
			addImage(count++, testFilter(src, new GlowFilter(0xFF0000), new Point(-10, -10)));
			addImage(count++, testFilter(src, new DropShadowFilter(4, 45, 0, 1, 4, 4), new Point(-10, -10)));
		}

		private function createCheckeredBackground(width:uint, height:uint):BitmapData {
			var bg:BitmapData = new BitmapData(width, height, false, 0xFFFFFFFF);
			for (var x:uint = 0; x < width; x += BG_CELL_SIZE) {
				for (var y:uint = 0; y < height; y += BG_CELL_SIZE) {
					var color:uint = 0xFFEEEEEE;
					if ((x / BG_CELL_SIZE + y / BG_CELL_SIZE) % 2 == 0) {
						color = 0xFFBBBBBB;
					}
					bg.fillRect(new Rectangle(x, y, BG_CELL_SIZE, BG_CELL_SIZE), color);
				}
			}
			return bg;
		}

		private function createSource():BitmapData {
			var src:BitmapData = new BitmapData(IMAGE_SIZE, IMAGE_SIZE, true, 0x00000000);

			for (var row:uint = 0; row < 10; row++) {
				for (var col:uint = 0; col < 10; col++) {
					var rd:Number = (row + 1) / 10;
					var cd:Number = (col + 1) / 10;
					var a:Number = 0.8;
					var r:Number = 1 - rd;
					var g:Number = rd;
					var b:Number = cd;
					var color:uint = ((int(a * 0xFF) & 0xFF) << 24) |
					                 ((int(r * 0xFF) & 0xFF) << 16) |
					                 ((int(g * 0xFF) & 0xFF) << 8) |
					                 ((int(b * 0xFF) & 0xFF) << 0);
					src.fillRect(
						new Rectangle(col * 10, row * 10, 10, 10),
						color
					);
				}
			}

			src.fillRect(new Rectangle(IMAGE_SIZE / 2 - 2, 5, 4, IMAGE_SIZE - 10), 0xFF000000);
			src.fillRect(new Rectangle(5, IMAGE_SIZE / 2 - 2, IMAGE_SIZE - 10, 4), 0xFF000000);

			return src;
		}

		private function addImage(count:uint, img:BitmapData):void {
			var bitmap:Bitmap = new Bitmap(img);
			var cols:uint = Math.floor(stage.stageWidth / IMAGE_SIZE);
			bitmap.x = (count % cols) * IMAGE_SIZE;
			bitmap.y = uint(count / cols) * IMAGE_SIZE;
			addChild(bitmap);
		}

		private function testFilter(src:BitmapData, filter:BitmapFilter, destPoint:Point):BitmapData {
			var dst:BitmapData = new BitmapData(IMAGE_SIZE, IMAGE_SIZE, true, 0x00000000);
			var sourceRect:Rectangle = new Rectangle(0, 0, IMAGE_SIZE, IMAGE_SIZE);
			dst.applyFilter(src, sourceRect, destPoint, filter);
			return dst;
		}
	}
}
