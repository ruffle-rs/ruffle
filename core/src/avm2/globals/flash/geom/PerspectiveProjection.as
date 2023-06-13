package flash.geom {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import __ruffle__.stub_setter;
    import flash.geom.Matrix3D;
    import flash.geom.Point;

    public class PerspectiveProjection {
        // Getters are stubbed with what seem to be Flash's default values
        public function PerspectiveProjection() {
            stub_constructor("flash.geom.PerspectiveProjection");
        }

        public function get fieldOfView():Number {
            stub_getter("flash.geom.PerspectiveProjection", "fieldOfView");
            return 55;
        }
        public function set fieldOfView(value:Number) {
            stub_setter("flash.geom.PerspectiveProjection", "fieldOfView");
        }

        public function get focalLength():Number {
            stub_getter("flash.geom.PerspectiveProjection", "focalLength");
            return 480.25;
        }
        public function set focalLength(value:Number) {
            stub_setter("flash.geom.PerspectiveProjection", "focalLength");
        }

        public function get projectionCenter():Point {
            stub_getter("flash.geom.PerspectiveProjection", "projectionCenter");
            return new Point(250, 250);
        }
        public function set projectionCenter(value:Point) {
            stub_setter("flash.geom.PerspectiveProjection", "projectionCenter");
        }

        public function toMatrix3D():Matrix3D {
            stub_method("flash.geom.PerspectiveProjection", "toMatrix3D");
            return new Matrix3D();
        }
    }
}