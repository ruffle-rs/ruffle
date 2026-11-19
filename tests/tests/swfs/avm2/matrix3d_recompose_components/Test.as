package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Orientation3D;
    import flash.geom.Vector3D;

    // Checks which components Matrix3D.recompose uses for which cells, by
    // replacing them with special values one or two at a time, along with
    // random components and the unit quaternion check.
    public class Test extends Sprite {
        private static const ORIENTATIONS:Array = [Orientation3D.EULER_ANGLES, Orientation3D.QUATERNION, Orientation3D.AXIS_ANGLE];

        // Translation, rotation and scale for each orientation.
        private static const BASES:Array = [
            [1, 2, 3, 0.1, 0.2, 0.3, 0.4, 1, 2, 0.5],
            [1, 2, 3, 0.1, 0.2, 0.3, 0.9273618495495703, 1, 2, 0.5],
            [1, 2, 3, 0.6, 0, 0.8, 0.5, 1, 2, 0.5]
        ];

        private static const NAMES:Array = ["translation.x", "translation.y", "translation.z",
            "rotation.x", "rotation.y", "rotation.z", "rotation.w", "scale.x", "scale.y", "scale.z"];

        private var seed:Number = 25235;

        public function Test() {
            var o:int;
            var i:int;
            var j:int;
            var a:Number;
            var b:Number;

            trace("// Single special values");
            for (o = 0; o < ORIENTATIONS.length; o++) {
                for (i = 0; i < NAMES.length; i++) {
                    for each (a in [NaN, Infinity, -Infinity, -0, 1e39, -1e39, 1e-45]) {
                        var single:Array = BASES[o].concat();
                        single[i] = a;
                        run(NAMES[i] + "=" + a, single, ORIENTATIONS[o]);
                    }
                }
            }

            trace("// Pairs of non-finite values");
            for (o = 0; o < ORIENTATIONS.length; o++) {
                for (i = 0; i < NAMES.length; i++) {
                    for (j = i + 1; j < NAMES.length; j++) {
                        for each (a in [NaN, Infinity, -Infinity]) {
                            for each (b in [NaN, Infinity, -Infinity]) {
                                var pair:Array = BASES[o].concat();
                                pair[i] = a;
                                pair[j] = b;
                                run(NAMES[i] + "=" + a + " " + NAMES[j] + "=" + b, pair, ORIENTATIONS[o]);
                            }
                        }
                    }
                }
            }

            trace("// Random components");
            for (i = 0; i < 100; i++) {
                var big:Boolean = random() < 0.2;
                randomRun([angle(big), angle(big), angle(big), uniform(-1, 1)], Orientation3D.EULER_ANGLES);
            }
            for (i = 0; i < 100; i++) {
                randomRun(unitQuaternion(), Orientation3D.QUATERNION);
            }
            for (i = 0; i < 100; i++) {
                var magnitude:Number = Math.pow(10, uniform(-3, 3));
                randomRun([uniform(-1, 1) * magnitude, uniform(-1, 1) * magnitude, uniform(-1, 1) * magnitude, angle(random() < 0.2)], Orientation3D.AXIS_ANGLE);
            }

            trace("// Quaternions around the unit length limits");
            for each (a in [0.9999949336051941, 0.9999949932098389, 1.0000050067901611, 1.0000051259994507]) {
                quaternionRun([0, 0, 0, a]);
            }

            trace("// Quaternions where the order of the squared length sum matters");
            quaternionRun([-0.7309941952895047, -0.03235054767069893, -0.06957598296242008, -0.6780487432956803]);
            quaternionRun([-0.4598564135514322, 0.6405267009342437, 0.6043782749282914, 0.1139054062998179]);
            quaternionRun([-0.24585481308768248, 0.5882936650146401, 0.41245209840067615, 0.6506451755130154]);
            quaternionRun([-0.0729986493169683, 0.13885366704486074, -0.4057385976621916, 0.900431655991661]);
        }

        private function random():Number {
            seed = (seed * 16807) % 2147483647;
            return seed / 2147483647;
        }

        private function uniform(min:Number, max:Number):Number {
            return min + (max - min) * random();
        }

        private function angle(big:Boolean):Number {
            return big ? uniform(-1000, 1000) : uniform(-7, 7);
        }

        private function unitQuaternion():Array {
            var q:Array = [uniform(-1, 1), uniform(-1, 1), uniform(-1, 1), uniform(-1, 1)];
            var length:Number = Math.sqrt(q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]);
            return [q[0] / length, q[1] / length, q[2] / length, q[3] / length];
        }

        private function randomRun(rotation:Array, orientation:String):void {
            var translation:Array = random() < 0.5 ? [0, 0, 0] : [uniform(-1000, 1000), uniform(-1000, 1000), uniform(-1000, 1000)];
            var scale:Array = [1, 1, 1];

            if (random() < 0.5) {
                for (var i:int = 0; i < 3; i++) {
                    scale[i] = uniform(0.1, 10) * (random() < 0.5 ? -1 : 1);
                }
            }

            var components:Array = translation.concat(rotation, scale);
            run(components.join(","), components, orientation);
        }

        private function quaternionRun(rotation:Array):void {
            var components:Array = [0, 0, 0].concat(rotation, [1, 1, 1]);
            run(rotation.join(","), components, Orientation3D.QUATERNION);
        }

        private function run(label:String, c:Array, orientation:String):void {
            var matrix:Matrix3D = new Matrix3D();
            var components:Vector.<Vector3D> = new <Vector3D>[
                new Vector3D(c[0], c[1], c[2]),
                new Vector3D(c[3], c[4], c[5], c[6]),
                new Vector3D(c[7], c[8], c[9])
            ];

            try {
                trace(orientation + " " + label + ": " + matrix.recompose(components, orientation) + " " + matrix.rawData);
            } catch (e:Error) {
                trace(orientation + " " + label + ": " + e + " " + matrix.rawData);
            }
        }
    }
}
