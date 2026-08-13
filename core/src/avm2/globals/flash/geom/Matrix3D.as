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
            this.rawData = Matrix3D.interpolate(this, toMat, percent).rawData;
        }

        public static function interpolate(thisMat:Matrix3D, toMat:Matrix3D, percent:Number):Matrix3D {
            var from:Vector.<Vector3D> = thisMat.decompose("quaternion");
            var to:Vector.<Vector3D> = toMat.decompose("quaternion");

            var t0:Vector3D = from[0], q0:Vector3D = from[1], s0:Vector3D = from[2];
            var t1:Vector3D = to[0], q1:Vector3D = to[1], s1:Vector3D = to[2];

            var trans:Vector3D = new Vector3D(
                t0.x + (t1.x - t0.x) * percent,
                t0.y + (t1.y - t0.y) * percent,
                t0.z + (t1.z - t0.z) * percent);

            var scale:Vector3D = new Vector3D(
                s0.x + (s1.x - s0.x) * percent,
                s0.y + (s1.y - s0.y) * percent,
                s0.z + (s1.z - s0.z) * percent);

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
            result.recompose(new <Vector3D>[trans, rot, scale], "quaternion");
            return result;
        }

    }
}
