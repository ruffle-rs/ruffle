package flash.geom {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import __ruffle__.stub_setter;
    import flash.display.DisplayObject;
    import flash.geom.Matrix3D;
    import flash.geom.Point;

    public class PerspectiveProjection {
        [Ruffle(NativeAccessible)]
        private var displayObject: DisplayObject = null;

        [Ruffle(NativeAccessible)]
        private var fov: Number = 55.0;

        [Ruffle(NativeAccessible)]
        private var center: Point = new Point(250, 250);

        public function PerspectiveProjection() {
        }

        public function get fieldOfView():Number {
            stub_getter("flash.geom.PerspectiveProjection", "fieldOfView");
            return this.fov;
        }

        public function set fieldOfView(value:Number) {
            // TODO: This setter should update the associated displayObject when there is.
            stub_setter("flash.geom.PerspectiveProjection", "fieldOfView");

            if (value <= 0 || 180 <= value) {
                throw new ArgumentError("Error #2182: Invalid fieldOfView value.  The value must be greater than 0 and less than 180.", 2182);
            }

            this.fov = value;
        }

        public native function get focalLength():Number;

        public native function set focalLength(value:Number);

        public function get projectionCenter():Point {
            stub_getter("flash.geom.PerspectiveProjection", "projectionCenter");
            return this.center;
        }
        public function set projectionCenter(value:Point) {
            // TODO: This setter should update the associated displayObject when there is.
            stub_setter("flash.geom.PerspectiveProjection", "projectionCenter");
            this.center = value;
        }

        public function toMatrix3D():Matrix3D {
            var fl: Number = this.focalLength;
            return new Matrix3D(new <Number>[
              fl, 0, 0, 0,
              0, fl, 0, 0,
              0, 0, 1, 1,
              0, 0, 0, 0
            ]);
        }
    }
}
