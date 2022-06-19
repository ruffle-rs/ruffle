package flash.geom {
	import flash.display.DisplayObject;
	public class Transform {
		private var _displayObject:DisplayObject;

		function Transform(object: DisplayObject) {
			this.init(object);
		}
		native function init(object:DisplayObject):void;

		public native function get colorTransform():ColorTransform;
		public native function set colorTransform(value: ColorTransform):void;
		public native function get matrix():Matrix;
		public native function set matrix(value:Matrix):void;

		public native function get concatenatedColorTransform():ColorTransform;
		public native function get concatenatedMatrix():Matrix;
	}
}
