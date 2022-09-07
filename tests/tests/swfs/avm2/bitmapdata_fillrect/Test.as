package {
	public class Test {
		public function Test() {
			import flash.display.BitmapData;
			import flash.geom.Rectangle;
			import flash.geom.Point;
			
			trace("Non-transparent BitmapData:");

			var background = 0;
			var data = new BitmapData(10, 10, false, background);
			var rect = new Rectangle(1, 1, 4, 5);
			var fill = 2;
			data.fillRect(rect, fill);
			for (var py = 0; py < 10; py++) {
				var line = "";
				for (var px = 0; px < 10; px++) {
					var actual = data.getPixel(px, py);
					line += actual + " ";
				}
				trace(line);
			}
			
			trace();
			
			printTransparent(0, 1);
			printTransparent(0xFFFFFFFF, 0);
		
		}
	}
}

function printTransparent(background:uint, fill:uint) {
	import flash.display.BitmapData;
	import flash.geom.Rectangle;
	import flash.geom.Point;
	
	trace("Transparent BitmapData: background=" + background + " fill=" + fill);
	var data = new BitmapData(10, 10, true, background);
	var rect = new Rectangle(1, 1, 4, 5);
	data.fillRect(rect, fill);
	for (var py = 0; py < 10; py++) {
		var line = "";
		for (var px = 0; px < 10; px++) {
			var actual = data.getPixel32(px, py);
			line += actual + " ";
		}
		trace(line);
	}
	trace();
}