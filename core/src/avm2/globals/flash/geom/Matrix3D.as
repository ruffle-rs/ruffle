// Based on the MIT-licensed OpenFL code https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx

package flash.geom {
    import __ruffle__.stub_method;

    public class Matrix3D {
        // The 4x4 matrix data, stored in column-major order
        // This is never null.
        [Ruffle(NativeAccessible)]
        private var _rawData:Vector.<Number>;

        public function get rawData():Vector.<Number> {
            return this._rawData.AS3::concat();
        }

        public function set rawData(value:Vector.<Number>):void {
            if (value != null) {
                this._rawData = value.AS3::concat();
            }
        }

        public function Matrix3D(v:Vector.<Number> = null) {
            if (v != null && v.length == 16) {
                this._rawData = v.AS3::concat();
            } else {
                this.identity();
            }
        }

        public function identity():void {
            // Note that every 4 elements is a *column*, not a row
            this._rawData = new <Number>[
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ];
        }

        public function appendTranslation(x:Number, y:Number, z:Number):void {
            this._rawData[12] += x;
            this._rawData[13] += y;
            this._rawData[14] += z;
        }

        public function appendRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
            var tx:Number, ty:Number, tz:Number;
            tx = ty = tz = 0;

            if (pivotPoint != null) {
                tx = pivotPoint.x;
                ty = pivotPoint.y;
                tz = pivotPoint.z;
            }
            var radian:Number = degrees * Math.PI / 180;
            var cos:Number = Math.cos(radian);
            var sin:Number = Math.sin(radian);
            var x:Number = axis.x;
            var y:Number = axis.y;
            var z:Number = axis.z;
            var x2:Number = x * x;
            var y2:Number = y * y;
            var z2:Number = z * z;
            var ls:Number = x2 + y2 + z2;
            if (ls != 0) {
                var l:Number = Math.sqrt(ls);
                x /= l;
                y /= l;
                z /= l;
                x2 /= ls;
                y2 /= ls;
                z2 /= ls;
            }
            var ccos:Number = 1 - cos;
            var m:Matrix3D = new Matrix3D();

            // Modify the matrix's data in-place
            var d:Vector.<Number> = m._rawData;
            d[0] = x2 + (y2 + z2) * cos;
            d[1] = x * y * ccos + z * sin;
            d[2] = x * z * ccos - y * sin;
            d[4] = x * y * ccos - z * sin;
            d[5] = y2 + (x2 + z2) * cos;
            d[6] = y * z * ccos + x * sin;
            d[8] = x * z * ccos + y * sin;
            d[9] = y * z * ccos - x * sin;
            d[10] = z2 + (x2 + y2) * cos;
            d[12] = (tx * (y2 + z2) - x * (ty * y + tz * z)) * ccos + (ty * z - tz * y) * sin;
            d[13] = (ty * (x2 + z2) - y * (tx * x + tz * z)) * ccos + (tz * x - tx * z) * sin;
            d[14] = (tz * (x2 + y2) - z * (tx * x + ty * y)) * ccos + (tx * y - ty * x) * sin;

            this.append(m);
        }

        [API("674")]
        public function copyRawDataFrom(vector:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void {
            if (transpose) {
                this.transpose();
            }

            var length = vector.length - index;

            for (var i = 0; i < length; i++) {
                this._rawData[i] = vector[i + index];
            }

            if (transpose) {
                this.transpose();
            }
        }

        [API("674")]
        public native function copyRowTo(row:uint, vector3D:Vector3D):void;

        [API("674")]
        public native function copyRowFrom(row:uint, vector3D:Vector3D):void;

        public native function deltaTransformVector(vector:Vector3D):Vector3D;

        public native function transformVector(vector:Vector3D):Vector3D;

        public native function transformVectors(vin:Vector.<Number>, vout:Vector.<Number>):void;

        [Ruffle(NativeCallable)]
        public native function transpose():void;

        public native function append(lhs:Matrix3D):void;

        // Based on https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L307
        public function appendScale(xScale:Number, yScale:Number, zScale:Number):void {
            this.append(new Matrix3D(Vector.<Number>([
                xScale, 0.0, 0.0, 0.0, 0.0, yScale, 0.0, 0.0, 0.0, 0.0, zScale, 0.0, 0.0, 0.0, 0.0, 1.0
            ])));
        }

        public function prependTranslation(x:Number, y:Number, z:Number):void {
            var m:Matrix3D = new Matrix3D();
            m.position = new Vector3D(x, y, z);
            this.prepend(m);
        }

        public function prependRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
            var m:Matrix3D = new Matrix3D();
            m.appendRotation(degrees, axis, pivotPoint);
            this.prepend(m);
        }

        public native function get position():Vector3D;
        public native function set position(val:Vector3D):void;

        public native function prepend(rhs:Matrix3D):void;

        public function prependScale(xScale:Number, yScale:Number, zScale:Number):void {
            var m:Matrix3D = new Matrix3D();
            m.appendScale(xScale, yScale, zScale);
            this.prepend(m);
        }

        [API("674")]
        public function copyFrom(other:Matrix3D):void {
            // This makes a copy of other.rawData
            this._rawData = other.rawData;
        }

        [API("674")]
        public function copyRawDataTo(vector:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void {
            if (transpose) {
                this.transpose();
            }

            var d:Vector.<Number> = this._rawData;
            for (var i = 0; i < d.length; i++) {
                vector[i + index] = d[i];
            }

            if (transpose) {
                this.transpose();
            }
        }

        [Ruffle(NativeCallable)]
        public function clone():Matrix3D {
            // The constructor will make a copy of this._rawData
            return new Matrix3D(this._rawData);
        }

        public function copyToMatrix3D(other:Matrix3D):void {
            // This makes a copy of this.rawData
            other._rawData = this.rawData;
        }

        public function pointAt(pos:Vector3D, at:Vector3D = null, up:Vector3D = null):void {
            stub_method("flash.geom.Matrix3D", "pointAt");
        }

        public native function recompose(components:Vector.<Vector3D>, orientationStyle:String = "eulerAngles"):Boolean;

        [API("674")]
        public native function copyColumnTo(column:uint, vector3D:Vector3D):void;

        [API("674")]
        public native function copyColumnFrom(column:uint, vector3D:Vector3D):void;

        public native function decompose(orientationStyle:String = "eulerAngles"):Vector.<Vector3D>;

        public native function invert():Boolean;

        public native function get determinant():Number;

    }
}
