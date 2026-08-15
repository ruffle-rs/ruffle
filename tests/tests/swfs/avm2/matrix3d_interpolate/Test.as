package {
    import flash.display.Sprite;
    import flash.geom.*;

    // Matrix3D.interpolate/interpolateTo interpolate the translation linearly
    // and the rotation via slerp of a quaternion read straight from the raw
    // matrix (scale is left in). The scale itself is discarded, so the result is
    // always unit-scaled. Every value is rounded to 6 decimals so tiny
    // floating-point noise doesn't affect the output.
    public class Test extends Sprite {
        public function Test() {
            // Translation is interpolated linearly.
            var t0:Matrix3D = translation(0, 0, 0);
            var t1:Matrix3D = translation(10, 20, 30);
            trace("translate p=0:   " + dump(Matrix3D.interpolate(t0, t1, 0)));
            trace("translate p=0.5: " + dump(Matrix3D.interpolate(t0, t1, 0.5)));
            trace("translate p=1:   " + dump(Matrix3D.interpolate(t0, t1, 1)));

            // Scale is discarded entirely: interpolating between two pure-scale
            // matrices always yields the identity.
            var s0:Matrix3D = scale(1, 1, 1);
            var s1:Matrix3D = scale(3, 5, 7);
            trace("scale p=0.5:     " + dump(Matrix3D.interpolate(s0, s1, 0.5)));

            // Rotation: slerp from identity to 90 degrees about Z, midpoint is a
            // 45 degree rotation (+/- sqrt(2)/2 = 0.707107).
            var r0:Matrix3D = new Matrix3D();
            var r1:Matrix3D = new Matrix3D();
            r1.appendRotation(90, Vector3D.Z_AXIS);
            trace("rotate p=0.5:    " + dump(Matrix3D.interpolate(r0, r1, 0.5)));
            trace("rotate p=1:      " + dump(Matrix3D.interpolate(r0, r1, 1)));

            // Rotation combined with a scale: because the quaternion is read from
            // the raw (scaled) matrix, the midpoint is not the naive 45 degrees.
            var rs0:Matrix3D = new Matrix3D();
            rs0.appendScale(2, 2, 2);
            var rs1:Matrix3D = new Matrix3D();
            rs1.appendScale(4, 4, 4);
            rs1.appendRotation(90, Vector3D.Z_AXIS);
            trace("rot+scale p=0.5: " + dump(Matrix3D.interpolate(rs0, rs1, 0.5)));

            // interpolate(A, A, t) == A when A has no scale.
            trace("self p=0.5:      " + dump(Matrix3D.interpolate(t1, t1, 0.5)));

            // interpolateTo mutates the receiver in place.
            var m:Matrix3D = translation(0, 0, 0);
            m.interpolateTo(translation(8, 8, 8), 0.25);
            trace("interpolateTo:   " + dump(m));

            // Null arguments throw TypeError #2007 naming the parameter.
            // Null arguments throw TypeError #2007 naming the parameter. The
            // stack traces match Flash's native methods frame-for-frame:
            // interpolateTo validates toMat itself, so it doesn't add an extra
            // interpolate() frame.
            try { Matrix3D.interpolate(null, m, 0.5); }
            catch (e:Error) { trace("interp(null,m):\n" + e.getStackTrace()); }
            try { Matrix3D.interpolate(m, null, 0.5); }
            catch (e:Error) { trace("interp(m,null):\n" + e.getStackTrace()); }
            try { m.interpolateTo(null, 0.5); }
            catch (e:Error) { trace("interpTo(null):\n" + e.getStackTrace()); }
        }

        function translation(x:Number, y:Number, z:Number):Matrix3D {
            var m:Matrix3D = new Matrix3D();
            m.appendTranslation(x, y, z);
            return m;
        }

        function scale(x:Number, y:Number, z:Number):Matrix3D {
            var m:Matrix3D = new Matrix3D();
            m.appendScale(x, y, z);
            return m;
        }

        function r6(n:Number):Number {
            return Math.round(n * 1e6) / 1e6;
        }

        function dump(m:Matrix3D):String {
            var d:Vector.<Number> = m.rawData;
            var out:Array = [];
            for (var i:int = 0; i < 16; i++) {
                out.push(r6(d[i]));
            }
            return out.join(",");
        }
    }
}
