package {
    import flash.display.Sprite;
    import flash.geom.*;

    // Matrix3D.interpolate/interpolateTo decompose both matrices into
    // (translation, rotation quaternion, scale), interpolate each component
    // (spherical linear interpolation for the rotation) and recompose the
    // result. Every value is rounded to 6 decimals so tiny floating-point
    // noise from the decompose/recompose round trip doesn't affect the output.
    public class Test extends Sprite {
        public function Test() {
            // Pure translation: the midpoint is the exact linear midpoint.
            var t0:Matrix3D = translation(0, 0, 0);
            var t1:Matrix3D = translation(10, 20, 30);
            trace("translate p=0:   " + dump(Matrix3D.interpolate(t0, t1, 0)));
            trace("translate p=0.5: " + dump(Matrix3D.interpolate(t0, t1, 0.5)));
            trace("translate p=1:   " + dump(Matrix3D.interpolate(t0, t1, 1)));

            // Pure scale: the midpoint is the exact linear midpoint.
            var s0:Matrix3D = scale(1, 1, 1);
            var s1:Matrix3D = scale(3, 5, 7);
            trace("scale p=0.5:     " + dump(Matrix3D.interpolate(s0, s1, 0.5)));

            // Rotation: slerp from identity to 90 degrees about Z, so the
            // midpoint is a 45 degree rotation (the two non-trivial entries
            // are +/- sqrt(2)/2 = 0.707107).
            var r0:Matrix3D = new Matrix3D();
            var r1:Matrix3D = new Matrix3D();
            r1.appendRotation(90, Vector3D.Z_AXIS);
            trace("rotate p=0.5:    " + dump(Matrix3D.interpolate(r0, r1, 0.5)));
            trace("rotate p=1:      " + dump(Matrix3D.interpolate(r0, r1, 1)));

            // interpolate(A, A, t) == A for any t.
            trace("self p=0.5:      " + dump(Matrix3D.interpolate(t1, t1, 0.5)));

            // interpolateTo mutates the receiver in place.
            var m:Matrix3D = translation(0, 0, 0);
            m.interpolateTo(translation(8, 8, 8), 0.25);
            trace("interpolateTo:   " + dump(m));
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
