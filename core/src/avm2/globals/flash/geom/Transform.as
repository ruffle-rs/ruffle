package flash.geom {
	import flash.display.DisplayObject;
	import flash.geom.Matrix3D;
	import flash.geom.PerspectiveProjection;
	import __ruffle__.stub_getter;
	import __ruffle__.stub_method;
	import __ruffle__.stub_setter;

	public class Transform {
	    [Ruffle(InternalSlot)]
		private var displayObject:DisplayObject;

		private var _matrix3D:Matrix3D = null;
		private var _perspectiveProjection:PerspectiveProjection = null;

		function Transform(object:DisplayObject) {
			this.displayObject = object;
		}

		public native function get colorTransform():ColorTransform;
		public native function set colorTransform(value:ColorTransform):void;
		public native function get matrix():Matrix;
		public native function set matrix(value:Matrix):void;

		public function get concatenatedColorTransform():ColorTransform {
			stub_getter("flash.geom.Transform", "concatenatedColorTransform");
			return new ColorTransform();
		}

		public native function get concatenatedMatrix():Matrix;
		public native function get pixelBounds():Rectangle;

		public function get matrix3D():Matrix3D {
			stub_getter("flash.geom.Transform", "matrix3D");
			return this._matrix3D;
		}

		public function set matrix3D(m:Matrix3D):void {
			stub_setter("flash.geom.Transform", "matrix3D");
			this._matrix3D = m;
		}

		public function get perspectiveProjection():PerspectiveProjection {
			stub_getter("flash.geom.Transform", "perspectiveProjection");
			return this._perspectiveProjection;
		}

		public function set perspectiveProjection(val: PerspectiveProjection):void {
			stub_setter("flash.geom.Transform", "perspectiveProjection");
			this._perspectiveProjection = val;
		}

		public function getRelativeMatrix3D(relativeTo:DisplayObject):Matrix3D {
			stub_method("flash.geom.Transform", "getRelativeMatrix3D");
			return new Matrix3D();
		}
	}
}
