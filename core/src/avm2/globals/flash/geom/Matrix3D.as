package flash.geom {
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class Matrix3D {
        public function Matrix3D(v:Vector.<Number> = null) {
            this.rawData = v;
        }

        public native function get rawData():Vector.<Number>;
        public native function set rawData(value:Vector.<Number>):void;

        public native function get position():Vector3D;
        public native function set position(val:Vector3D):void;

        public native function identity():void;

        public native function transpose():void;

        public native function get determinant():Number;

        public native function invert():Boolean;

        public native function append(lhs:Matrix3D):void;
        public native function appendTranslation(x:Number, y:Number, z:Number):void;
        public native function appendScale(xScale:Number, yScale:Number, zScale:Number):void;
        public native function appendRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void;

        public native function prepend(rhs:Matrix3D):void;
        public native function prependTranslation(x:Number, y:Number, z:Number):void;
        public native function prependScale(xScale:Number, yScale:Number, zScale:Number):void;
        public function prependRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
            var m:Matrix3D = new Matrix3D();
            m.appendRotation(degrees, axis, pivotPoint);
            this.prepend(m);
        }

        public native function deltaTransformVector(vector:Vector3D):Vector3D;
        public native function transformVector(vector:Vector3D):Vector3D;
        public native function transformVectors(vin:Vector.<Number>, vout:Vector.<Number>):void;

        public native function decompose(orientationStyle:String = "eulerAngles"):Vector.<Vector3D>;
        public native function recompose(components:Vector.<Vector3D>, orientationStyle:String = "eulerAngles"):Boolean;

        [API("674")]
        public native function copyRowFrom(row:uint, vector3D:Vector3D):void;
        [API("674")]
        public native function copyRowTo(row:uint, vector3D:Vector3D):void;
        [API("674")]
        public native function copyColumnFrom(column:uint, vector3D:Vector3D):void;
        [API("674")]
        public native function copyColumnTo(column:uint, vector3D:Vector3D):void;

        [API("674")]
        public native function copyRawDataFrom(source:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void;
        [API("674")]
        public native function copyRawDataTo(dest:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void;

        [API("674")]
        public native function copyFrom(source:Matrix3D):void;
        public native function copyToMatrix3D(dest:Matrix3D):void;

        public native function clone():Matrix3D;

        public function pointAt(pos:Vector3D, at:Vector3D = null, up:Vector3D = null):void {
            stub_method("flash.geom.Matrix3D", "pointAt");
        }
    }
}
