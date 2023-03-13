package {
	import flash.display.MovieClip;
	import flash.display.Shape;
	import flash.geom.Matrix;
	import flash.geom.Point;
	
	public class Test {
		public function Test(main:MovieClip) {
			main.stage.scaleMode = "noScale";
			
			trace("NOTE: This test MUST be run under a (possibly simulated) display with a scale factor of 2");
			trace("stage.transform.matrix: " + main.stage.transform.matrix);
			trace("stage.contentsScaleFactor: " + main.stage.contentsScaleFactor);
			
			trace("stage.transform.matrix: " + main.stage.transform.matrix);
			trace("stage.contentsScaleFactor: " + main.stage.contentsScaleFactor);
			trace("stage.localToGlobal(new Point(0, 0)) = " + main.stage.localToGlobal(new Point(0, 0)));
			
			var rect = new Shape();
			rect.graphics.beginFill(0x0);
			rect.graphics.drawRect(0, 0, 200, 200);
			rect.graphics.endFill();
			
			main.stage.transform.matrix = new Matrix(1.5, 0, 0, 1, 10, 100);
			trace("Set stage matrix");
			
			var full = new Shape();
			full.graphics.beginFill(0x00FF00, 0.2);
			full.graphics.drawRect(0, 0, main.stage.stageWidth, main.stage.stageHeight);
			full.graphics.endFill();
			
			trace("stage.transform.matrix: " + main.stage.transform.matrix);
			trace("stage.contentsScaleFactor: " + main.stage.contentsScaleFactor);			
			trace("stage.localToGlobal(new Point(0, 0)) = " + main.stage.localToGlobal(new Point(0, 0)));
			
			main.stage.addChild(rect);
			main.stage.addChild(full);
			
			main.stage.addEventListener("mouseDown", function(e) {
				trace("mouseDown: stageX= " + e.stageX + " stageY=" + e.stageY + " localX=" + e.localX + " localY = " + e.localY);
			})
		
			new Context3D_drawTriangles(main.stage);
		}
	}
}