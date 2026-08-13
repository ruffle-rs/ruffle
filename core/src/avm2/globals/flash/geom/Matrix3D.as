// Based on the MIT-licensed OpenFL code https://github.com/openfl/openfl/blob/develop/src/openfl/geom/Matrix3D.hx

package flash.geom {
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class Matrix3D {
        public function Matrix3D(v:Vector.<Number> = null) {
            if (v != null && v.length == 16) {
                this.rawData = v;
            }
        }

        public native function get rawData():Vector.<Number>;
        public native function set rawData(value:Vector.<Number>):void;

        public native function identity():void;

        public native function appendTranslation(x:Number, y:Number, z:Number):void;

        public native function appendRotation(degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null):void;

        [API("674")]
        public native function copyRawDataFrom(source:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void;

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
            this.rawData = other.rawData;
        }

        [API("674")]
        public native function copyRawDataTo(dest:Vector.<Number>, index:uint = 0, transpose:Boolean = false):void;

        [Ruffle(NativeCallable)]
        public function clone():Matrix3D {
            return new Matrix3D(this.rawData);
        }

        public function copyToMatrix3D(other:Matrix3D):void {
            other.rawData = this.rawData;
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

        public function interpolateTo(toMat:Matrix3D, percent:Number):void {
            if (toMat == null) {
                throw new TypeError("Error #2007: Parameter toMat must be non-null.", 2007);
            }
            this.rawData = Matrix3D.interpolate(this, toMat, percent).rawData;
        }

        public static function interpolate(fromMat:Matrix3D, toMat:Matrix3D, percent:Number):Matrix3D {
            if (fromMat == null) {
                throw new TypeError("Error #2007: Parameter fromMat must be non-null.", 2007);
            }
            if (toMat == null) {
                throw new TypeError("Error #2007: Parameter toMat must be non-null.", 2007);
            }

            var a:Vector.<Number> = fromMat.rawData;
            var b:Vector.<Number> = toMat.rawData;

            // Flash reads the rotation quaternion straight from the raw matrix
            // (scale is left in, which is why a rotation combined with a scale
            // does not slerp to the naive angle) and discards the scale
            // entirely. Only the translation and the normalized rotation are
            // interpolated.
            var q0:Vector3D = quaternionOf(a);
            var q1:Vector3D = quaternionOf(b);

            var trans:Vector3D = new Vector3D(
                a[12] + (b[12] - a[12]) * percent,
                a[13] + (b[13] - a[13]) * percent,
                a[14] + (b[14] - a[14]) * percent);

            // Spherical linear interpolation of the rotation quaternions.
            var dot:Number = q0.x * q1.x + q0.y * q1.y + q0.z * q1.z + q0.w * q1.w;
            var x1:Number = q1.x, y1:Number = q1.y, z1:Number = q1.z, w1:Number = q1.w;
            if (dot < 0) {
                dot = -dot;
                x1 = -x1; y1 = -y1; z1 = -z1; w1 = -w1;
            }

            var k0:Number, k1:Number;
            if (dot > 0.9995) {
                k0 = 1 - percent;
                k1 = percent;
            } else {
                var theta:Number = Math.acos(dot);
                var sinTheta:Number = Math.sin(theta);
                k0 = Math.sin((1 - percent) * theta) / sinTheta;
                k1 = Math.sin(percent * theta) / sinTheta;
            }

            var rx:Number = q0.x * k0 + x1 * k1;
            var ry:Number = q0.y * k0 + y1 * k1;
            var rz:Number = q0.z * k0 + z1 * k1;
            var rw:Number = q0.w * k0 + w1 * k1;

            var len:Number = Math.sqrt(rx * rx + ry * ry + rz * rz + rw * rw);
            if (len == 0) {
                rx = 0; ry = 0; rz = 0; rw = 1; len = 1;
            }
            var rot:Vector3D = new Vector3D(rx / len, ry / len, rz / len, rw / len);

            var result:Matrix3D = new Matrix3D();
            result.recompose(new <Vector3D>[trans, rot, new Vector3D(1, 1, 1)], "quaternion");
            return result;
        }

        // Normalized rotation quaternion of a raw column-major 4x4 matrix, taken
        // from its upper-left 3x3 with the scale left in.
        private static function quaternionOf(m:Vector.<Number>):Vector3D {
            var m00:Number = m[0], m10:Number = m[1], m20:Number = m[2];
            var m01:Number = m[4], m11:Number = m[5], m21:Number = m[6];
            var m02:Number = m[8], m12:Number = m[9], m22:Number = m[10];

            var x:Number, y:Number, z:Number, w:Number, s:Number;
            var trace:Number = m00 + m11 + m22;
            if (trace > 0) {
                s = Math.sqrt(trace + 1) * 2;
                w = 0.25 * s;
                x = (m21 - m12) / s;
                y = (m02 - m20) / s;
                z = (m10 - m01) / s;
            } else if (m00 > m11 && m00 > m22) {
                s = Math.sqrt(1 + m00 - m11 - m22) * 2;
                w = (m21 - m12) / s;
                x = 0.25 * s;
                y = (m01 + m10) / s;
                z = (m02 + m20) / s;
            } else if (m11 > m22) {
                s = Math.sqrt(1 + m11 - m00 - m22) * 2;
                w = (m02 - m20) / s;
                x = (m01 + m10) / s;
                y = 0.25 * s;
                z = (m12 + m21) / s;
            } else {
                s = Math.sqrt(1 + m22 - m00 - m11) * 2;
                w = (m10 - m01) / s;
                x = (m02 + m20) / s;
                y = (m12 + m21) / s;
                z = 0.25 * s;
            }

            var len:Number = Math.sqrt(x * x + y * y + z * z + w * w);
            if (len == 0) {
                return new Vector3D(0, 0, 0, 1);
            }
            return new Vector3D(x / len, y / len, z / len, w / len);
        }

    }
}
