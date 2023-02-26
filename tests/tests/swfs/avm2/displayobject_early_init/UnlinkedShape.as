package {
	import flash.display.Shape;

	public class UnlinkedShape extends Shape {
		public function UnlinkedShape() {
			trace("UnlinkedShape before super(): this.graphics: " + this.graphics);
			super();
			trace("UnlinkedShape after super(): this.graphics: " + this.graphics);
		}
	}
}