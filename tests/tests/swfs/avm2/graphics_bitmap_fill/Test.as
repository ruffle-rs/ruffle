package {
	import flash.display.*;
    import flash.geom.*;

	public class Test extends MovieClip {
		[Embed (source="logo.png")]
		public static const LogoBitmap:Class;

		public function Test() {
			super();

			var logoBitmap = new LogoBitmap();
            var bmd = logoBitmap.bitmapData;

            var matrix: Matrix = new Matrix();
            matrix.scale(1, 1);

			use_objects(bmd, matrix);
			use_functions(bmd, matrix);
		}

		public function create_rect_path(base: Point): GraphicsPath {
			var points = [
			    base.add(new Point(0, 0)),
			    base.add(new Point(0, 150)),
			    base.add(new Point(150, 150)),
			    base.add(new Point(150, 0))
			];

            var path: GraphicsPath = new GraphicsPath();
            path.moveTo(points[0].x, points[0].y);
            for (var i = 1; i < points.length; i++) {
            	path.lineTo(points[i].x, points[i].y);
            }
            path.lineTo(points[0].x, points[0].y);

            return path;
        }

		public function use_objects(bmd: BitmapData, mtx: Matrix): void {
            var child: Shape = new Shape();

            var fill: GraphicsBitmapFill = new GraphicsBitmapFill(bmd, mtx);

            child.graphics.drawGraphicsData(Vector.<IGraphicsData>([
                fill,
                create_rect_path(new Point()),
                new GraphicsEndFill()
            ]));

            var stroke: GraphicsStroke = new GraphicsStroke(5);
            stroke.fill = fill;

            child.graphics.drawGraphicsData(Vector.<IGraphicsData>([
                stroke,
                create_rect_path(new Point(200, 0))
            ]));

            addChild(child);
		}

		public function use_functions(bmd: BitmapData, mtx: Matrix): void {
		    var child: Shape = new Shape();

            child.graphics.beginBitmapFill(bmd, mtx);
            child.graphics.drawRect(0, 0, 150, 150);
            child.graphics.endFill();

            child.graphics.lineStyle(5);
            child.graphics.lineBitmapStyle(bmd, mtx);
            child.graphics.drawRect(200, 0, 150, 150);

            addChild(child);
            child.y = 200;
		}
	}
}
