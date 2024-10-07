// Based on the MIT-licensed OpenFL code https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx

package flash.geom {
    import __ruffle__.stub_method;

    public class Matrix3D {

        // The 4x4 matrix data, stored in column-major order
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
            var radian = degrees * Math.PI / 180;
            var cos = Math.cos(radian);
            var sin = Math.sin(radian);
            var x = axis.x;
            var y = axis.y;
            var z = axis.z;
            var x2 = x * x;
            var y2 = y * y;
            var z2 = z * z;
            var ls = x2 + y2 + z2;
            if (ls != 0) {
                var l = Math.sqrt(ls);
                x /= l;
                y /= l;
                z /= l;
                x2 /= ls;
                y2 /= ls;
                z2 /= ls;
            }
            var ccos = 1 - cos;
            var m = new Matrix3D();

            var d = m.rawData;
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
            m.rawData = d;

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
                    vector3D.x = rawData[0];
                    vector3D.y = rawData[4];
                    vector3D.z = rawData[8];
                    vector3D.w = rawData[12];
                    break;
                case 1:
                    vector3D.x = rawData[1];
                    vector3D.y = rawData[5];
                    vector3D.z = rawData[9];
                    vector3D.w = rawData[13];
                    break;
                case 2:
                    vector3D.x = rawData[2];
                    vector3D.y = rawData[6];
                    vector3D.z = rawData[10];
                    vector3D.w = rawData[14];
                    break;
                case 3:
                    vector3D.x = rawData[3];
                    vector3D.y = rawData[7];
                    vector3D.z = rawData[11];
                    vector3D.w = rawData[15];
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

        public function transpose():void {
            // Make a copy
            var oRawData = this._rawData.AS3::concat();
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
        public function append(lhs:Matrix3D):void {
            var m111:Number = this._rawData[0],
                m121:Number = this._rawData[4],
                m131:Number = this._rawData[8],
                m141:Number = this._rawData[12],
                m112:Number = this._rawData[1],
                m122:Number = this._rawData[5],
                m132:Number = this._rawData[9],
                m142:Number = this._rawData[13],
                m113:Number = this._rawData[2],
                m123:Number = this._rawData[6],
                m133:Number = this._rawData[10],
                m143:Number = this._rawData[14],
                m114:Number = this._rawData[3],
                m124:Number = this._rawData[7],
                m134:Number = this._rawData[11],
                m144:Number = this._rawData[15],
                m211:Number = lhs._rawData[0],
                m221:Number = lhs._rawData[4],
                m231:Number = lhs._rawData[8],
                m241:Number = lhs._rawData[12],
                m212:Number = lhs._rawData[1],
                m222:Number = lhs._rawData[5],
                m232:Number = lhs._rawData[9],
                m242:Number = lhs._rawData[13],
                m213:Number = lhs._rawData[2],
                m223:Number = lhs._rawData[6],
                m233:Number = lhs._rawData[10],
                m243:Number = lhs._rawData[14],
                m214:Number = lhs._rawData[3],
                m224:Number = lhs._rawData[7],
                m234:Number = lhs._rawData[11],
                m244:Number = lhs._rawData[15];

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

        // Based on https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L307
        public function appendScale(xScale:Number, yScale:Number, zScale:Number):void {
            this.append(new Matrix3D(Vector.<Number>([
                    xScale, 0.0, 0.0, 0.0, 0.0, yScale, 0.0, 0.0, 0.0, 0.0, zScale, 0.0, 0.0, 0.0, 0.0, 1.0
                ])));
        }

        public function prependTranslation(x:Number, y:Number, z:Number):void {
            var m = new Matrix3D();
            m.position = new Vector3D(x, y, z);
            this.prepend(m);
        }

        public function prependRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void {
            var m = new Matrix3D();
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
            var m = new Matrix3D();
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

            for (var i = 0; i < rawData.length; i++) {
                vector[i + index] = _rawData[i];
            }

            if (transpose) {
                this.transpose();
            }
        }

        public function clone():Matrix3D {
            return new Matrix3D(this.rawData);
        }

        public function copyToMatrix3D(other:Matrix3D):void {
            other.rawData = rawData;
        }

        public function pointAt(pos:Vector3D, at:Vector3D = null, up:Vector3D = null):void {
            stub_method("flash.geom.Matrix3D", "pointAt");
        }

        // Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L1437
        public function recompose(components:Vector.<Vector3D>, orientationStyle:String = "eulerAngles"):Boolean {
            checkOrientation(orientationStyle);

            if (orientationStyle == Orientation3D.QUATERNION) {
                // Flash throws exceptions from 'recompose' certain values of 'components',
                // which we need to reproduce. See the 'matrix3d_compose' test
                stub_method("flash.geom.Matrix3D", "recompose", "Orientation3D.QUATERNION");
            }
            // RUFFLE - unlike in OpenFL, we continue on even if some of the 'scale' components are 0
            if (components.length < 3) {
                return false;
            }

            identity();

            var scale = [];
            scale[0] = scale[1] = scale[2] = components[2].x;
            scale[4] = scale[5] = scale[6] = components[2].y;
            scale[8] = scale[9] = scale[10] = components[2].z;

            switch (orientationStyle) {
                case Orientation3D.EULER_ANGLES:
                    var cx = Math.cos(components[1].x);
                    var cy = Math.cos(components[1].y);
                    var cz = Math.cos(components[1].z);
                    var sx = Math.sin(components[1].x);
                    var sy = Math.sin(components[1].y);
                    var sz = Math.sin(components[1].z);

                    _rawData[0] = cy * cz * scale[0];
                    _rawData[1] = cy * sz * scale[1];
                    _rawData[2] = -sy * scale[2];
                    _rawData[3] = 0;
                    _rawData[4] = (sx * sy * cz - cx * sz) * scale[4];
                    _rawData[5] = (sx * sy * sz + cx * cz) * scale[5];
                    _rawData[6] = sx * cy * scale[6];
                    _rawData[7] = 0;
                    _rawData[8] = (cx * sy * cz + sx * sz) * scale[8];
                    _rawData[9] = (cx * sy * sz - sx * cz) * scale[9];
                    _rawData[10] = cx * cy * scale[10];
                    _rawData[11] = 0;
                    _rawData[12] = components[0].x;
                    _rawData[13] = components[0].y;
                    _rawData[14] = components[0].z;
                    _rawData[15] = 1;
                    break;

                default:
                    var x = components[1].x;
                    var y = components[1].y;
                    var z = components[1].z;
                    var w = components[1].w;

                    if (orientationStyle == Orientation3D.AXIS_ANGLE) {
                        x *= Math.sin(w / 2);
                        y *= Math.sin(w / 2);
                        z *= Math.sin(w / 2);
                        w = Math.cos(w / 2);
                    }

                    _rawData[0] = (1 - 2 * y * y - 2 * z * z) * scale[0];
                    _rawData[1] = (2 * x * y + 2 * w * z) * scale[1];
                    _rawData[2] = (2 * x * z - 2 * w * y) * scale[2];
                    _rawData[3] = 0;
                    _rawData[4] = (2 * x * y - 2 * w * z) * scale[4];
                    _rawData[5] = (1 - 2 * x * x - 2 * z * z) * scale[5];
                    _rawData[6] = (2 * y * z + 2 * w * x) * scale[6];
                    _rawData[7] = 0;
                    _rawData[8] = (2 * x * z + 2 * w * y) * scale[8];
                    _rawData[9] = (2 * y * z - 2 * w * x) * scale[9];
                    _rawData[10] = (1 - 2 * x * x - 2 * y * y) * scale[10];
                    _rawData[11] = 0;
                    _rawData[12] = components[0].x;
                    _rawData[13] = components[0].y;
                    _rawData[14] = components[0].z;
                    _rawData[15] = 1;
            }

            if (components[2].x == 0) {
                _rawData[0] = 1e-15;
            }

            if (components[2].y == 0) {
                _rawData[5] = 1e-15;
            }

            if (components[2].z == 0) {
                _rawData[10] = 1e-15;
            }

            return !(components[2].x == 0 || components[2].y == 0 || components[2].y == 0);
        }

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

        public function decompose(orientationStyle:String = "eulerAngles"):Vector.<Vector3D> {
            checkOrientation(orientationStyle);

            var vec = new Vector.<Vector3D>([]);
            var m = clone();
            var mr = m.rawData;

            var pos = new Vector3D(mr[12], mr[13], mr[14]);
            mr[12] = 0;
            mr[13] = 0;
            mr[14] = 0;

            var scale = new Vector3D();

            scale.x = Math.sqrt(mr[0] * mr[0] + mr[1] * mr[1] + mr[2] * mr[2]);
            scale.y = Math.sqrt(mr[4] * mr[4] + mr[5] * mr[5] + mr[6] * mr[6]);
            scale.z = Math.sqrt(mr[8] * mr[8] + mr[9] * mr[9] + mr[10] * mr[10]);

            if (mr[0] * (mr[5] * mr[10] - mr[6] * mr[9]) - mr[1] * (mr[4] * mr[10] - mr[6] * mr[8]) + mr[2] * (mr[4] * mr[9] - mr[5] * mr[8]) < 0) {
                scale.z = -scale.z;
            }

            mr[0] /= scale.x;
            mr[1] /= scale.x;
            mr[2] /= scale.x;
            mr[4] /= scale.y;
            mr[5] /= scale.y;
            mr[6] /= scale.y;
            mr[8] /= scale.z;
            mr[9] /= scale.z;
            mr[10] /= scale.z;

            var rot = new Vector3D();

            switch (orientationStyle) {
                case Orientation3D.AXIS_ANGLE:
                    rot.w = Math.acos((mr[0] + mr[5] + mr[10] - 1) / 2);

                    var len = Math.sqrt((mr[6] - mr[9]) * (mr[6] - mr[9]) + (mr[8] - mr[2]) * (mr[8] - mr[2]) + (mr[1] - mr[4]) * (mr[1] - mr[4]));

                    if (len != 0) {
                        rot.x = (mr[6] - mr[9]) / len;
                        rot.y = (mr[8] - mr[2]) / len;
                        rot.z = (mr[1] - mr[4]) / len;
                    }
                    else {
                        rot.x = rot.y = rot.z = 0;
                    }
                    break;

                case Orientation3D.QUATERNION:
                    var tr = mr[0] + mr[5] + mr[10];

                    if (tr > 0) {
                        rot.w = Math.sqrt(1 + tr) / 2;

                        rot.x = (mr[6] - mr[9]) / (4 * rot.w);
                        rot.y = (mr[8] - mr[2]) / (4 * rot.w);
                        rot.z = (mr[1] - mr[4]) / (4 * rot.w);
                    }
                    else if ((mr[0] > mr[5]) && (mr[0] > mr[10])) {
                        rot.x = Math.sqrt(1 + mr[0] - mr[5] - mr[10]) / 2;

                        rot.w = (mr[6] - mr[9]) / (4 * rot.x);
                        rot.y = (mr[1] + mr[4]) / (4 * rot.x);
                        rot.z = (mr[8] + mr[2]) / (4 * rot.x);
                    }
                    else if (mr[5] > mr[10]) {
                        rot.y = Math.sqrt(1 + mr[5] - mr[0] - mr[10]) / 2;

                        rot.x = (mr[1] + mr[4]) / (4 * rot.y);
                        rot.w = (mr[8] - mr[2]) / (4 * rot.y);
                        rot.z = (mr[6] + mr[9]) / (4 * rot.y);
                    }
                    else {
                        rot.z = Math.sqrt(1 + mr[10] - mr[0] - mr[5]) / 2;

                        rot.x = (mr[8] + mr[2]) / (4 * rot.z);
                        rot.y = (mr[6] + mr[9]) / (4 * rot.z);
                        rot.w = (mr[1] - mr[4]) / (4 * rot.z);
                    }
                    break;

                case Orientation3D.EULER_ANGLES:
                    rot.y = Math.asin(-mr[2]);

                    if (mr[2] != 1 && mr[2] != -1) {
                        rot.x = Math.atan2(mr[6], mr[10]);
                        rot.z = Math.atan2(mr[1], mr[0]);
                    }
                    else {
                        rot.z = 0;
                        rot.x = Math.atan2(mr[4], mr[5]);
                    }
                    break;
            }

            vec.push(pos);
            vec.push(rot);
            vec.push(scale);

            return vec;
        }

        public function invert():Boolean {
            var d = determinant;
            var invertable = Math.abs(d) > 0.00000000001;

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

import flash.geom.Orientation3D;

function checkOrientation(orientationStyle:String) {
    if (!(orientationStyle == Orientation3D.AXIS_ANGLE || orientationStyle == Orientation3D.EULER_ANGLES || orientationStyle == Orientation3D.QUATERNION)) {
        throw new Error("Error #2187: Invalid orientation style " + orientationStyle + ".  Value must be one of 'Orientation3D.EULER_ANGLES', 'Orientation3D.AXIS_ANGLE', or 'Orientation3D.QUATERNION'.", 2187);
    }
}
