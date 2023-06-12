package flash.geom {
	import flash.display.DisplayObject;
	import __ruffle__.stub_getter;
	import __ruffle__.stub_method;
	import __ruffle__.stub_setter;

	public class Transform {
		internal var _displayObject:DisplayObject;

		function Transform(object:DisplayObject) {
			this.init(object);
		}
		native function init(object:DisplayObject):void;

		public native function get colorTransform():ColorTransform;
		public native function set colorTransform(value:ColorTransform):void;
		public native function get matrix():Matrix;
		public native function set matrix(value:Matrix):void;

		public native function get concatenatedColorTransform():ColorTransform;
		public native function get concatenatedMatrix():Matrix;
		public native function get pixelBounds():Rectangle;

		public function get matrix3D():Matrix3D {
			stub_getter("flash.geom.Transform", "matrix3D");
			return null;
		}

		public function set matrix3D(m:Matrix3D):void {
			stub_setter("flash.geom.Transform", "matrix3D");
		}

		public function get perspectiveProjection():PerspectiveProjection {
			stub_getter("flash.geom.Transform", "perspectiveProjection");
			return null;
		}

		public function set perspectiveProjection(val: PerspectiveProjection):void {
			stub_setter("flash.geom.Transform", "perspectiveProjection");
		}

		public function getRelativeMatrix3D(relativeTo:DisplayObject):Matrix3D {
			stub_method("flash.geom.Transform", "getRelativeMatrix3D");
			return null;
		}
	}
}
