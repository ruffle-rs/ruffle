package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Orientation3D;
    import flash.geom.Vector3D;

    // Checks the results of Matrix3D.decompose for all orientations: random
    // matrices (also with skew and perspective cells), special values in each
    // cell, mirroring, gimbal lock, ties between the quaternion branches,
    // scales close to 1, tiny rotations and extreme scales. The matrices are
    // only built from literals and basic arithmetic, so that they are the
    // same in any player.
    public class Test extends Sprite {
        private static const BASE:Array = [0.8, 0.3, -0.2, 0, -0.4, 1.7, 0.5, 0, 0.3, -0.6, 2.2, 0, 10, 20, 30, 1];
        private static const MIRRORED:Array = [-0.8, 0.3, -0.2, 0, -0.4, 1.7, 0.5, 0, 0.3, -0.6, 2.2, 0, 10, 20, 30, 1];

        private var seed:Number = 25235;

        public function Test() {
            randomMatrices();
            specialValues();
            mirroring();
            gimbalLock();
            quaternionBranches();
            scalesCloseToOne();
            tinyRotations();
            extremeScales();
        }

        private function randomMatrices():void {
            trace("// Random matrices");
            for (var i:int = 0; i < 150; i++) {
                run("random", randomMatrix(false));
            }

            trace("// Random matrices with perspective cells");
            for (i = 0; i < 100; i++) {
                run("random", randomMatrix(true));
            }
        }

        private function specialValues():void {
            trace("// Special values in each cell");
            for (var cell:int = 0; cell < 16; cell++) {
                for each (var value:Number in [NaN, Infinity, -Infinity, -0, 2]) {
                    var raw:Array = BASE.concat();
                    raw[cell] = value;
                    run("m" + cell + "=" + value, raw);
                }
            }
        }

        private function mirroring():void {
            trace("// Mirrored matrices with perspective cells");
            for each (var cells:Array in [[3], [7], [11], [3, 7], [3, 11], [7, 11], [3, 7, 11]]) {
                for each (var value:Number in [0.0001, 0.001, 0.003, 0.005, 0.01, 0.1, -1]) {
                    var raw:Array = MIRRORED.concat();
                    for each (var cell:int in cells) {
                        raw[cell] = value;
                    }
                    run("m" + cells.join("+m") + "=" + value, raw);
                }
            }
        }

        private function gimbalLock():void {
            trace("// Gimbal lock");
            run("m2=-1", [0, 0, -1, 0, 0.6, 0.8, 0, 0, 0.8, -0.6, 0, 0, 0, 0, 0, 1]);
            run("m2=1", [0, 0, 1, 0, 0.6, 0.8, 0, 0, -0.8, 0.6, 0, 0, 0, 0, 0, 1]);
            for each (var m2:Number in [0.99999994, 0.9999999, 0.999999, -0.99999994, -0.9999999, -0.999999]) {
                var c:Number = Math.sqrt(1 - m2 * m2);
                run("m2=" + m2, [c, 0, m2, 0, 0, 1, 0, 0, -m2, 0, c, 0, 0, 0, 0, 1]);
            }
        }

        private function quaternionBranches():void {
            trace("// Ties between the quaternion branches");
            for each (var d:Array in [[1, -1, -1], [-1, 1, -1], [-1, -1, 1], [-1, -1, -1], [1, 1, -1], [0.5, 0.5, -1], [-0.5, -0.5, 1]]) {
                run("diagonal " + d.join(","), [d[0], 0, 0, 0, 0, d[1], 0, 0, 0, 0, d[2], 0, 0, 0, 0, 1]);
            }
            run("permutation 1", [0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1]);
            run("permutation 2", [0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1]);
            run("permutation 3", [0, 1, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 1]);
            run("half turn 1", [-0.6, 0.8, 0, 0, 0.8, 0.6, 0, 0, 0, 0, -1, 0, 0, 0, 0, 1]);
            run("half turn 2", [-0.28, 0, 0.96, 0, 0, -1, 0, 0, 0.96, 0, 0.28, 0, 0, 0, 0, 1]);
        }

        private function scalesCloseToOne():void {
            trace("// Scales close to 1");
            for each (var base:Number in [0.999, 1.001]) {
                for (var i:int = -15; i <= 15; i++) {
                    var s:Number = base + i * 1.2e-8;
                    run("scale " + s, [s, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0.5, 0, 0, 0, 0, 1]);
                }
            }
        }

        private function tinyRotations():void {
            trace("// Tiny rotations");
            for each (var size:Number in [1e-1, 1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7, 1e-8, 1e-9, 0]) {
                for (var i:int = 0; i < 3; i++) {
                    var x:Number = size * uniform(-1, 1);
                    var y:Number = size * uniform(-1, 1);
                    var z:Number = size * uniform(-1, 1);
                    var w:Number = Math.sqrt(1 - x * x - y * y - z * z);
                    run("rotation " + size, quaternionMatrix(x, y, z, w));
                }
            }
        }

        private function extremeScales():void {
            trace("// Extreme scales");
            for each (var s:Array in [[1e-20, 1, 1], [1, 1e20, 1], [1, 1, 1e-30], [1e20, 1e-20, 1], [-1e15, 1e-15, 1e10]]) {
                var raw:Array = quaternionMatrix(0.1, 0.2, 0.3, Math.sqrt(0.86));
                for (var i:int = 0; i < 3; i++) {
                    for (var k:int = 0; k < 3; k++) {
                        raw[i * 4 + k] *= s[i];
                    }
                }
                run("scale " + s.join(","), raw);
            }
        }

        private function quaternionMatrix(x:Number, y:Number, z:Number, w:Number):Array {
            return [
                1 - 2 * (y * y + z * z), 2 * (x * y + w * z), 2 * (x * z - w * y), 0,
                2 * (x * y - w * z), 1 - 2 * (x * x + z * z), 2 * (y * z + w * x), 0,
                2 * (x * z + w * y), 2 * (y * z - w * x), 1 - 2 * (x * x + y * y), 0,
                0, 0, 0, 1
            ];
        }

        private function randomMatrix(perspective:Boolean):Array {
            var raw:Array = [];
            for (var k:int = 0; k < 16; k++) {
                if (k == 3 || k == 7 || k == 11) {
                    raw.push(perspective ? uniform(-1, 1) : 0);
                } else if (k == 15) {
                    raw.push(1);
                } else {
                    raw.push(k >= 12 ? uniform(-1000, 1000) : uniform(-3, 3));
                }
            }
            return raw;
        }

        private function uniform(min:Number, max:Number):Number {
            seed = (seed * 16807) % 2147483647;
            return min + (max - min) * seed / 2147483647;
        }

        private static function cell(v:Number):String {
            return v == 0 && 1 / v < 0 ? "-0" : String(v);
        }

        private static function vector(v:Vector3D, withW:Boolean):String {
            return cell(v.x) + "," + cell(v.y) + "," + cell(v.z) + (withW ? "," + cell(v.w) : "");
        }

        private function run(label:String, raw:Array):void {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(raw));
            trace(label);

            for each (var orientation:String in [Orientation3D.EULER_ANGLES, Orientation3D.QUATERNION, Orientation3D.AXIS_ANGLE]) {
                var components:Vector.<Vector3D> = matrix.decompose(orientation);
                // Flash Player leaves rotation.w uninitialized for Euler angles.
                var withW:Boolean = orientation != Orientation3D.EULER_ANGLES;
                trace("  " + orientation + ": " + vector(components[0], false) + " | " + vector(components[1], withW) + " | " + vector(components[2], false));
            }
        }
    }
}
