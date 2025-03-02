package flash.geom {
    import flash.display.DisplayObject;
    import flash.geom.Matrix3D;
    import flash.geom.Point;

    public class PerspectiveProjection {
        [Ruffle(NativeAccessible)]
        private var displayObject:DisplayObject = null;

        [Ruffle(NativeAccessible)]
        private var fov:Number = 55.0;

        [Ruffle(NativeAccessible)]
        private var center:Point = new Point(250, 250);

        public function PerspectiveProjection() {
        }

        public native function get fieldOfView():Number;

        public native function set fieldOfView(value:Number);

        public native function get focalLength():Number;

        public native function set focalLength(value:Number);

        public native function get projectionCenter():Point;

        public native function set projectionCenter(value:Point);

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
