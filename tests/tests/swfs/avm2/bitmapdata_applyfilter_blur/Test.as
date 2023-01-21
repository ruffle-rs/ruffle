package {
	import flash.display.DisplayObjectContainer;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.filters.BlurFilter;
	import flash.filters.BitmapFilter;
	import flash.geom.Point;
	import flash.geom.Rectangle;
	import flash.geom.Matrix;
	import flash.text.TextField;

	public class Test {
		static var BG_CELL_SIZE: uint = 9;

		static var SOURCE_WIDTH: uint = 220;
		static var SOURCE_HEIGHT: uint = 220;

		static var GRID_ROWS: uint = 11;
		static var GRID_COLS: uint = 11;

		public static function run(container: DisplayObjectContainer) {
			var bg: BitmapData = createCheckeredBackground(container.stage.stageWidth, container.stage.stageHeight);
			container.addChild(new Bitmap(bg));

			var src: BitmapData = createSource();
			var count: uint = 0;

			addImage(container, count++, src);

			var matrix;

			addImage(container, count++, testFilter(src, new BlurFilter()));

			addImage(container, count++, testFilter(src, new BlurFilter(3, 3, 4)));
			addImage(container, count++, testFilter(src, new BlurFilter(10, 1, 15)));
			addImage(container, count++, testFilter(src, new BlurFilter(1, 10, 1)));
			addImage(container, count++, testFilter(src, new BlurFilter(5, 5, 3)));
		}

		static function createCheckeredBackground(width: uint, height: uint): BitmapData {
			var bg: BitmapData = new BitmapData(width, height, false);
			for (var x: uint = 0; x < width; x += BG_CELL_SIZE) {
				for (var y: uint = 0; y < height; y += BG_CELL_SIZE) {
					var color: uint = 0xFFEEEEEE;
					if ((x / BG_CELL_SIZE + y / BG_CELL_SIZE) % 2 == 0) {
						color = 0xFFBBBBBB;
					}
					bg.fillRect(new Rectangle(x, y, BG_CELL_SIZE, BG_CELL_SIZE), color);
				}
			}
			return bg;
		}

		static function createSource(): BitmapData {
			var src: BitmapData = new BitmapData(SOURCE_WIDTH, SOURCE_HEIGHT, true, 0x00000000);
			for (var row:uint = 0; row < GRID_ROWS; row++) {
				for (var col:uint = 0; col < GRID_COLS; col++) {
					var rd: Number = (row + 1) / GRID_ROWS;
					var cd: Number = (col + 1) / GRID_COLS;
					var a: Number = 1 - Math.pow(((1 - rd) + cd) / 2, 2);
					var r: Number = 1 - rd;
					var g: Number = rd;
					var b: Number = cd;
					var color: uint = ((int(a * 0xFF) & 0xFF) << 24) | ((int(r * 0xFF) & 0xFF) << 16) | ((int(g * 0xFF) & 0xFF) << 8) | ((int(b * 0xFF) & 0xFF) << 0);
					src.fillRect(new Rectangle(col * (SOURCE_WIDTH / GRID_COLS), row * (SOURCE_HEIGHT / GRID_ROWS), SOURCE_WIDTH / GRID_COLS, SOURCE_HEIGHT / GRID_ROWS), color);
				}
			}
			src.fillRect(new Rectangle((SOURCE_WIDTH / 2) - 3, 3, 6, SOURCE_HEIGHT - 6), 0xFF000000);
			src.fillRect(new Rectangle(3, (SOURCE_HEIGHT / 2) - 3, SOURCE_WIDTH - 6, 6), 0xFF000000);


			var text: TextField = new TextField();
			text.text = "Ruffle Ruffle Ruffle Ruffle Ruffle Ruffle Ruffle Ruffle Ruffle";
			text.width = SOURCE_WIDTH;
			text.height = 20;
			var mat:Matrix = new Matrix();
			mat.translate(0, SOURCE_HEIGHT / 2 - 20);
			src.draw(text, mat);

			return src;
		}

		static function addImage(container: DisplayObjectContainer, count: uint, img: BitmapData) {
			var bitmap: Bitmap = new Bitmap(img);
			var rows: uint = Math.floor(container.stage.stageWidth / SOURCE_WIDTH);
			bitmap.x = (count % rows) * SOURCE_WIDTH;
			bitmap.y = uint(count / rows) * SOURCE_HEIGHT;
			container.addChild(bitmap);
		}

		static function testFilter(src: BitmapData, filter: BitmapFilter): BitmapData {
			var dst: BitmapData = new BitmapData(SOURCE_WIDTH, SOURCE_HEIGHT, true, 0x00000000);
			var point: Point = new Point(5, 5);
			var sourceRect: Rectangle = new Rectangle(10, 10, SOURCE_WIDTH - 20, SOURCE_HEIGHT - 20);

			//dst["applyFilter"](src, sourceRect, );
			dst.applyFilter(src, sourceRect, point, filter);

			return dst;
		}
	}
}
