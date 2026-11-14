package flash.geom {
    import flash.display.DisplayObject;

    public class Transform {
        [Ruffle(NativeAccessible)]
        private var displayObject:DisplayObject;

        public function Transform(displayObject:DisplayObject) {
            this.displayObject = displayObject;
        }

        public native function get colorTransform():ColorTransform;
        public native function set colorTransform(value:ColorTransform):void;

        public native function get concatenatedColorTransform():ColorTransform;

        public native function get concatenatedMatrix():Matrix;

        public native function get matrix():Matrix;
        public native function set matrix(value:Matrix):void;

        public native function get pixelBounds():Rectangle;

        [API("662")]
        public native function get matrix3D():Matrix3D;
        [API("662")]
        public native function set matrix3D(m:Matrix3D):void;

        [API("662")]
        public native function get perspectiveProjection():PerspectiveProjection;
        [API("662")]
        public native function set perspectiveProjection(pm:PerspectiveProjection):void;

        [API("662")]
        public native function getRelativeMatrix3D(relativeTo:DisplayObject):Matrix3D;
    }
}
