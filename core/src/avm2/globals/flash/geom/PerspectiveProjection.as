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

        public function PerspectiveProjection() {}

        public native function get fieldOfView():Number;

        public native function set fieldOfView(value:Number):void;

        public native function get focalLength():Number;

        public native function set focalLength(value:Number):void;

        public native function get projectionCenter():Point;

        public native function set projectionCenter(value:Point):*;

        public native function toMatrix3D():Matrix3D;
    }
}
