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

        // Based on https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx#L542C1-L573
        [API("674")]
        public function copyRowTo(row:uint, vector3D:Vector3D):void {
            if (row > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            switch (row) {
                case 0:
                    vector3D.x = _rawData[0];
                    vector3D.y = _rawData[4];
                    vector3D.z = _rawData[8];
                    vector3D.w = _rawData[12];
                    break;
                case 1:
                    vector3D.x = _rawData[1];
                    vector3D.y = _rawData[5];
                    vector3D.z = _rawData[9];
                    vector3D.w = _rawData[13];
                    break;
                case 2:
                    vector3D.x = _rawData[2];
                    vector3D.y = _rawData[6];
                    vector3D.z = _rawData[10];
                    vector3D.w = _rawData[14];
                    break;
                case 3:
                    vector3D.x = _rawData[3];
                    vector3D.y = _rawData[7];
                    vector3D.z = _rawData[11];
                    vector3D.w = _rawData[15];
                    break;
            }
        }

        // Based on https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx#L504-L534
        [API("674")]
        public function copyRowFrom(row:uint, vector3D:Vector3D):void {
            if (row > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            switch (row) {
                case 0:
                    _rawData[0] = vector3D.x;
                    _rawData[4] = vector3D.y;
                    _rawData[8] = vector3D.z;
                    _rawData[12] = vector3D.w;
                    break;
                case 1:
                    _rawData[1] = vector3D.x;
                    _rawData[5] = vector3D.y;
                    _rawData[9] = vector3D.z;
                    _rawData[13] = vector3D.w;
                    break;
                case 2:
                    _rawData[2] = vector3D.x;
                    _rawData[6] = vector3D.y;
                    _rawData[10] = vector3D.z;
                    _rawData[14] = vector3D.w;
                    break;
                case 3:
                    _rawData[3] = vector3D.x;
                    _rawData[7] = vector3D.y;
                    _rawData[11] = vector3D.z;
                    _rawData[15] = vector3D.w;
                    break;
            }
        }

        public function deltaTransformVector(v:Vector3D):Vector3D {
            var x:Number = this._rawData[0] * v.x + this._rawData[4] * v.y + this._rawData[8] * v.z;
            var y:Number = this._rawData[1] * v.x + this._rawData[5] * v.y + this._rawData[9] * v.z;
            var z:Number = this._rawData[2] * v.x + this._rawData[6] * v.y + this._rawData[10] * v.z;
            var w:Number = this._rawData[3] * v.x + this._rawData[7] * v.y + this._rawData[11] * v.z;
            return new Vector3D(x, y, z, w);
        }

        public function transformVector(v:Vector3D):Vector3D {
            var x:Number = this._rawData[0] * v.x + this._rawData[4] * v.y + this._rawData[8] * v.z + this._rawData[12];
            var y:Number = this._rawData[1] * v.x + this._rawData[5] * v.y + this._rawData[9] * v.z + this._rawData[13];
            var z:Number = this._rawData[2] * v.x + this._rawData[6] * v.y + this._rawData[10] * v.z + this._rawData[14];
            var w:Number = this._rawData[3] * v.x + this._rawData[7] * v.y + this._rawData[11] * v.z + this._rawData[15];
            return new Vector3D(x, y, z, w);
        }

        public function transformVectors(vin:Vector.<Number>, vout:Vector.<Number>):void {
            if (vin == null) {
                throw new TypeError("Error #2007: Parameter vin must be non-null.", 2007);
            }
            if (vout == null) {
                throw new TypeError("Error #2007: Parameter vout must be non-null.", 2007);
            }

            var resultVecsLength:Number = Math.floor(vin.length / 3) * 3;
            if (resultVecsLength > vout.length && vout.fixed) {
                throw new RangeError("Error #1126: Cannot change the length of a fixed Vector.")
            }

            var result3D:Vector3D;
            for (var i = 0; i < resultVecsLength; i += 3) {
                result3D = transformVector(new Vector3D(vin[i], vin[i + 1], vin[i + 2]));
                if (i <= vout.length) {
                    vout[i] = result3D.x;
                    vout[i + 1] = result3D.y;
                    vout[i + 2] = result3D.z;
                } else {
                    vout.push(result3D.x, result3D.y, result3D.z);
                }
            }
        }

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
