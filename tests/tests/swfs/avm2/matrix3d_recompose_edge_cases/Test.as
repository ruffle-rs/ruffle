package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testNullComponents();
            testZeroScale();
            testNegativeScale();
            testComponentCount();
            testOrientationStyle();
            testQuaternion();
            testAxisAngle();
            testSpecialValues();
        }

        // Checks how the 'orientationStyle' argument is coerced.
        function testOrientationStyle() {
            var components:Vector.<Vector3D> = Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(0, 0, 0), new Vector3D(7, 8, 9)
            ]);

            for each (var style in [5, true, "", undefined]) {
                testRecompose("style " + style, components, style);
            }
        }

        // A negative scale mirrors the matrix.
        function testNegativeScale() {
            testRecompose("negative scale", Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(0, 0, 0), new Vector3D(-7, 8, -9)
            ]));
        }

        // Checks how many components are used, and whether the ones past
        // the third one matter at all.
        function testComponentCount() {
            var all:Array = [
                new Vector3D(1, 2, 3),
                new Vector3D(0, 0, 0),
                new Vector3D(7, 8, 9),
                null
            ];

            for each (var count in [0, 1, 2, 3, 4]) {
                var components:Vector.<Vector3D> = new Vector.<Vector3D>();
                for (var i = 0; i < count; i++) {
                    components.push(all[i]);
                }

                testRecompose("count " + count, components);
            }
        }

        // Flash validates the rotation component when using quaternions,
        // and throws for some of its values. All of these rotations produce
        // an exactly representable matrix.
        function testQuaternion() {
            for each (var rotation in [
                // Unit quaternions
                new Vector3D(0, 0, 0, 1),
                new Vector3D(0, 0, 0, -1),
                new Vector3D(1, 0, 0, 0),
                new Vector3D(0.5, 0.5, 0.5, 0.5),
                new Vector3D(-0.5, 0.5, -0.5, 0.5),
                // Not unit quaternions
                new Vector3D(0, 0, 0, 0),
                new Vector3D(0.5, 0, 0, 0),
                new Vector3D(2, 0, 0, 0),
                new Vector3D(1, 1, 1, 1),
                // Not finite, so not unit ones either
                new Vector3D(NaN, 0, 0, 1),
                new Vector3D(0, 0, 0, NaN),
                new Vector3D(Infinity, 0, 0, 0)
            ]) {
                testRotation(rotation, Orientation3D.QUATERNION);
            }
        }

        // A degenerate axis or angle.
        function testAxisAngle() {
            for each (var rotation in [
                new Vector3D(0, 0, 0, 0),
                // Non-zero axis, zero angle
                new Vector3D(1, 2, 3, 0),
                // Not finite
                new Vector3D(NaN, 0, 0, 0),
                new Vector3D(0, 0, 0, NaN),
                new Vector3D(0, 0, 0, Infinity)
            ]) {
                testRotation(rotation, Orientation3D.AXIS_ANGLE);
            }
        }

        // NaN and infinities in the components.
        function testSpecialValues() {
            testRecompose("NaN scale", Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(0, 0, 0), new Vector3D(NaN, 1, 1)
            ]));
            testRecompose("infinite scale", Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(0, 0, 0), new Vector3D(Infinity, 1, 1)
            ]));
            testRecompose("NaN rotation", Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(NaN, 0, 0), new Vector3D(1, 1, 1)
            ]));
            testRecompose("infinite rotation", Vector.<Vector3D>([
                new Vector3D(1, 2, 3), new Vector3D(Infinity, 0, 0), new Vector3D(1, 1, 1)
            ]));
            testRecompose("special translation", Vector.<Vector3D>([
                new Vector3D(Infinity, -Infinity, NaN), new Vector3D(0, 0, 0), new Vector3D(1, 1, 1)
            ]));
        }

        function testRotation(rotation:Vector3D, orientationStyle:String) {
            var label:String = "(" + rotation.x + ", " + rotation.y + ", "
                + rotation.z + ", " + rotation.w + ")";
            var components:Vector.<Vector3D> = Vector.<Vector3D>([
                new Vector3D(1, 2, 3),
                rotation,
                new Vector3D(1, 1, 1)
            ]);

            testRecompose(label, components, orientationStyle);
        }

        function testRecompose(label:String, components:Vector.<Vector3D>,
                               orientationStyle:String = "eulerAngles") {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            trace("Recompose " + label + " with style " + orientationStyle + ":");
            try {
                trace("Recompose res: " + matrix.recompose(components, orientationStyle));
            } catch (e) {
                trace("Caught error: " + e.getStackTrace());
            }
            trace("Recomposed: " + matrix.rawData);
        }

        // Checks the return value and the resulting matrix when only some
        // of the scale components are 0.
        function testZeroScale() {
            for each (var scale in [
                new Vector3D(7, 8, 9),
                new Vector3D(0, 8, 9),
                new Vector3D(7, 0, 9),
                new Vector3D(7, 8, 0),
                new Vector3D(0, 0, 9),
                new Vector3D(0, 8, 0),
                new Vector3D(7, 0, 0),
                new Vector3D(0, 0, 0)
            ]) {
                var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
                ]));
                var components:Vector.<Vector3D> = Vector.<Vector3D>([
                    new Vector3D(1, 2, 3),
                    new Vector3D(0, 0, 0),
                    scale
                ]);

                trace("Scale: " + scale);
                trace("Recompose res: " + matrix.recompose(components));
                trace("Recomposed: " + matrix.rawData);
            }
        }

        // Checks what's left in the matrix when one of the 'components'
        // is null, and 'recompose' throws part-way through.
        function testNullComponents() {
            for each (var nullIndex in [0, 1, 2]) {
                var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
                ]));
                var components:Vector.<Vector3D> = Vector.<Vector3D>([
                    new Vector3D(1, 2, 3),
                    new Vector3D(4, 5, 6),
                    new Vector3D(7, 8, 9)
                ]);
                components[nullIndex] = null;

                trace("Null component: " + nullIndex);
                try {
                    trace("Recompose res: " + matrix.recompose(components));
                } catch (e) {
                    trace("Caught error: " + e.getStackTrace());
                }
                trace("Recomposed: " + matrix.rawData);
            }
        }

        function testExceptions() {
            testException(function() {
                new Matrix3D().recompose(null, Orientation3D.EULER_ANGLES);
            });
            testException(function() {
                new Matrix3D().recompose(Vector.<Vector3D>([]), null);
            });
            testException(function() {
                new Matrix3D().recompose(Vector.<Vector3D>([]), "EulerAngles");
            });
        }

        function testException(f:Function) {
            try {
                f();
                trace("Didn't throw");
            } catch (e) {
                trace("Caught error: " + e.getStackTrace());
            }
        }
    }
}
