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
        public function transpose():void {
            // This makes a copy of this.rawData
            var oRawData:Vector.<Number> = this.rawData;
            this._rawData[1] = oRawData[4];
            this._rawData[2] = oRawData[8];
            this._rawData[3] = oRawData[12];
            this._rawData[4] = oRawData[1];
            this._rawData[6] = oRawData[9];
            this._rawData[7] = oRawData[13];
            this._rawData[8] = oRawData[2];
            this._rawData[9] = oRawData[6];
            this._rawData[11] = oRawData[14];
            this._rawData[12] = oRawData[3];
            this._rawData[13] = oRawData[7];
            this._rawData[14] = oRawData[11];
        }

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

        public function get position():Vector3D {
            return new Vector3D(_rawData[12], _rawData[13], _rawData[14]);
        }

        public function set position(val:Vector3D):void {
            this._rawData[12] = val.x;
            this._rawData[13] = val.y;
            this._rawData[14] = val.z;
        }

        public function prepend(rhs:Matrix3D):void {
            var m111:Number = rhs._rawData[0],
                m121:Number = rhs._rawData[4],
                m131:Number = rhs._rawData[8],
                m141:Number = rhs._rawData[12],
                m112:Number = rhs._rawData[1],
                m122:Number = rhs._rawData[5],
                m132:Number = rhs._rawData[9],
                m142:Number = rhs._rawData[13],
                m113:Number = rhs._rawData[2],
                m123:Number = rhs._rawData[6],
                m133:Number = rhs._rawData[10],
                m143:Number = rhs._rawData[14],
                m114:Number = rhs._rawData[3],
                m124:Number = rhs._rawData[7],
                m134:Number = rhs._rawData[11],
                m144:Number = rhs._rawData[15],
                m211:Number = this._rawData[0],
                m221:Number = this._rawData[4],
                m231:Number = this._rawData[8],
                m241:Number = this._rawData[12],
                m212:Number = this._rawData[1],
                m222:Number = this._rawData[5],
                m232:Number = this._rawData[9],
                m242:Number = this._rawData[13],
                m213:Number = this._rawData[2],
                m223:Number = this._rawData[6],
                m233:Number = this._rawData[10],
                m243:Number = this._rawData[14],
                m214:Number = this._rawData[3],
                m224:Number = this._rawData[7],
                m234:Number = this._rawData[11],
                m244:Number = this._rawData[15];

            this._rawData[0] = m111 * m211 + m112 * m221 + m113 * m231 + m114 * m241;
            this._rawData[1] = m111 * m212 + m112 * m222 + m113 * m232 + m114 * m242;
            this._rawData[2] = m111 * m213 + m112 * m223 + m113 * m233 + m114 * m243;
            this._rawData[3] = m111 * m214 + m112 * m224 + m113 * m234 + m114 * m244;

            this._rawData[4] = m121 * m211 + m122 * m221 + m123 * m231 + m124 * m241;
            this._rawData[5] = m121 * m212 + m122 * m222 + m123 * m232 + m124 * m242;
            this._rawData[6] = m121 * m213 + m122 * m223 + m123 * m233 + m124 * m243;
            this._rawData[7] = m121 * m214 + m122 * m224 + m123 * m234 + m124 * m244;

            this._rawData[8] = m131 * m211 + m132 * m221 + m133 * m231 + m134 * m241;
            this._rawData[9] = m131 * m212 + m132 * m222 + m133 * m232 + m134 * m242;
            this._rawData[10] = m131 * m213 + m132 * m223 + m133 * m233 + m134 * m243;
            this._rawData[11] = m131 * m214 + m132 * m224 + m133 * m234 + m134 * m244;

            this._rawData[12] = m141 * m211 + m142 * m221 + m143 * m231 + m144 * m241;
            this._rawData[13] = m141 * m212 + m142 * m222 + m143 * m232 + m144 * m242;
            this._rawData[14] = m141 * m213 + m142 * m223 + m143 * m233 + m144 * m243;
            this._rawData[15] = m141 * m214 + m142 * m224 + m143 * m234 + m144 * m244;
        }

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
        public function copyColumnTo(column:uint, vector3D:Vector3D):void {
            if (column > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            switch (column) {
                case 0:
                    vector3D.x = _rawData[0];
                    vector3D.y = _rawData[1];
                    vector3D.z = _rawData[2];
                    vector3D.w = _rawData[3];
                    break;

                case 1:
                    vector3D.x = _rawData[4];
                    vector3D.y = _rawData[5];
                    vector3D.z = _rawData[6];
                    vector3D.w = _rawData[7];
                    break;

                case 2:
                    vector3D.x = _rawData[8];
                    vector3D.y = _rawData[9];
                    vector3D.z = _rawData[10];
                    vector3D.w = _rawData[11];
                    break;

                case 3:
                    vector3D.x = _rawData[12];
                    vector3D.y = _rawData[13];
                    vector3D.z = _rawData[14];
                    vector3D.w = _rawData[15];
                    break;
            }
        }

        [API("674")]
        public function copyColumnFrom(column:uint, vector3D:Vector3D):void {
            if (column > 3) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }
            switch (column) {
                case 0:
                    _rawData[0] = vector3D.x;
                    _rawData[1] = vector3D.y;
                    _rawData[2] = vector3D.z;
                    _rawData[3] = vector3D.w;
                    break;

                case 1:
                    _rawData[4] = vector3D.x;
                    _rawData[5] = vector3D.y;
                    _rawData[6] = vector3D.z;
                    _rawData[7] = vector3D.w;
                    break;

                case 2:
                    _rawData[8] = vector3D.x;
                    _rawData[9] = vector3D.y;
                    _rawData[10] = vector3D.z;
                    _rawData[11] = vector3D.w;
                    break;

                case 3:
                    _rawData[12] = vector3D.x;
                    _rawData[13] = vector3D.y;
                    _rawData[14] = vector3D.z;
                    _rawData[15] = vector3D.w;
                    break;
            }
        }

        public native function decompose(orientationStyle:String = "eulerAngles"):Vector.<Vector3D>;

        public function invert():Boolean {
            var d:Number = this.determinant;
            var invertable:Boolean = Math.abs(d) > 0.00000000001;

            if (invertable) {
                d = 1 / d;

                var m11:Number = _rawData[0];
                var m21:Number = _rawData[4];
                var m31:Number = _rawData[8];
                var m41:Number = _rawData[12];
                var m12:Number = _rawData[1];
                var m22:Number = _rawData[5];
                var m32:Number = _rawData[9];
                var m42:Number = _rawData[13];
                var m13:Number = _rawData[2];
                var m23:Number = _rawData[6];
                var m33:Number = _rawData[10];
                var m43:Number = _rawData[14];
                var m14:Number = _rawData[3];
                var m24:Number = _rawData[7];
                var m34:Number = _rawData[11];
                var m44:Number = _rawData[15];

                _rawData[0] = d * (m22 * (m33 * m44 - m43 * m34) - m32 * (m23 * m44 - m43 * m24) + m42 * (m23 * m34 - m33 * m24));
                _rawData[1] = -d * (m12 * (m33 * m44 - m43 * m34) - m32 * (m13 * m44 - m43 * m14) + m42 * (m13 * m34 - m33 * m14));
                _rawData[2] = d * (m12 * (m23 * m44 - m43 * m24) - m22 * (m13 * m44 - m43 * m14) + m42 * (m13 * m24 - m23 * m14));
                _rawData[3] = -d * (m12 * (m23 * m34 - m33 * m24) - m22 * (m13 * m34 - m33 * m14) + m32 * (m13 * m24 - m23 * m14));
                _rawData[4] = -d * (m21 * (m33 * m44 - m43 * m34) - m31 * (m23 * m44 - m43 * m24) + m41 * (m23 * m34 - m33 * m24));
                _rawData[5] = d * (m11 * (m33 * m44 - m43 * m34) - m31 * (m13 * m44 - m43 * m14) + m41 * (m13 * m34 - m33 * m14));
                _rawData[6] = -d * (m11 * (m23 * m44 - m43 * m24) - m21 * (m13 * m44 - m43 * m14) + m41 * (m13 * m24 - m23 * m14));
                _rawData[7] = d * (m11 * (m23 * m34 - m33 * m24) - m21 * (m13 * m34 - m33 * m14) + m31 * (m13 * m24 - m23 * m14));
                _rawData[8] = d * (m21 * (m32 * m44 - m42 * m34) - m31 * (m22 * m44 - m42 * m24) + m41 * (m22 * m34 - m32 * m24));
                _rawData[9] = -d * (m11 * (m32 * m44 - m42 * m34) - m31 * (m12 * m44 - m42 * m14) + m41 * (m12 * m34 - m32 * m14));
                _rawData[10] = d * (m11 * (m22 * m44 - m42 * m24) - m21 * (m12 * m44 - m42 * m14) + m41 * (m12 * m24 - m22 * m14));
                _rawData[11] = -d * (m11 * (m22 * m34 - m32 * m24) - m21 * (m12 * m34 - m32 * m14) + m31 * (m12 * m24 - m22 * m14));
                _rawData[12] = -d * (m21 * (m32 * m43 - m42 * m33) - m31 * (m22 * m43 - m42 * m23) + m41 * (m22 * m33 - m32 * m23));
                _rawData[13] = d * (m11 * (m32 * m43 - m42 * m33) - m31 * (m12 * m43 - m42 * m13) + m41 * (m12 * m33 - m32 * m13));
                _rawData[14] = -d * (m11 * (m22 * m43 - m42 * m23) - m21 * (m12 * m43 - m42 * m13) + m41 * (m12 * m23 - m22 * m13));
                _rawData[15] = d * (m11 * (m22 * m33 - m32 * m23) - m21 * (m12 * m33 - m32 * m13) + m31 * (m12 * m23 - m22 * m13));
            }

            return invertable;
        }

        public function get determinant():Number {
            return 1 * ((_rawData[0] * _rawData[5] - _rawData[4] * _rawData[1]) * (_rawData[10] * _rawData[15] - _rawData[14] * _rawData[11])
                - (_rawData[0] * _rawData[9] - _rawData[8] * _rawData[1]) * (_rawData[6] * _rawData[15] - _rawData[14] * _rawData[7])
                + (_rawData[0] * _rawData[13] - _rawData[12] * _rawData[1]) * (_rawData[6] * _rawData[11] - _rawData[10] * _rawData[7])
                + (_rawData[4] * _rawData[9] - _rawData[8] * _rawData[5]) * (_rawData[2] * _rawData[15] - _rawData[14] * _rawData[3])
                - (_rawData[4] * _rawData[13] - _rawData[12] * _rawData[5]) * (_rawData[2] * _rawData[11] - _rawData[10] * _rawData[3])
                + (_rawData[8] * _rawData[13] - _rawData[12] * _rawData[9]) * (_rawData[2] * _rawData[7] - _rawData[6] * _rawData[3]));
        }

    }
}
