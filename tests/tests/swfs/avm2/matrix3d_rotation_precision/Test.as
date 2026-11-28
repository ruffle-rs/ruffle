package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        private var seed:Number = 12345;

        public function Test() {
            var i:int;

            // Angles where the ways of converting degrees to radians differ.
            var angles:Array = [90, 180, -270, 360, 45, 80819.99940369901, -52919.82900978229, 27179.99961441444, 1e20, 1e-300];
            for (i = 0; i < angles.length; i++) {
                testAll(angles[i], new Vector3D(0, 0, 1));
            }

            // Axes that aren't normalized, are zero, or overflow or underflow at f32 precision.
            var axes:Array = [
                new Vector3D(0, 3, 4), new Vector3D(1, 1, 1), new Vector3D(0, 0, 0), new Vector3D(-0, -0, 0),
                new Vector3D(1e-23, 0, 0), new Vector3D(1e-20, 1e-20, 1e-20), new Vector3D(1e20, 1e20, 1e20),
                new Vector3D(1e39, 0, 0), new Vector3D(NaN, 0, 1)
            ];
            for (i = 0; i < axes.length; i++) {
                testAll(33.3, axes[i]);
            }

            testAll(NaN, new Vector3D(1, 0, 0));
            testAll(Infinity, new Vector3D(1, 0, 0));

            for (i = 0; i < 25; i++) {
                var scale:Number = random() < 0.5 ? 1 : uniform(0.001, 1000);
                var axis:Vector3D = new Vector3D(scale * uniform(-1, 1), scale * uniform(-1, 1), scale * uniform(-1, 1));
                testAll(uniform(-720, 720), axis);
            }
        }

        private function testAll(degrees:Number, axis:Vector3D):void {
            var start:Vector.<Number> = new Vector.<Number>();
            for (var i:int = 0; i < 16; i++) {
                start.push(uniform(-4, 4));
            }
            var pivot:Vector3D = new Vector3D(uniform(-100, 100), uniform(-100, 100), uniform(-100, 100));

            trace("degrees " + degrees + ", axis " + axis + ", pivot " + pivot);
            test("append", degrees, axis, null, null);
            test("append", degrees, axis, pivot, null);
            test("append", degrees, axis, pivot, start);
            test("prepend", degrees, axis, null, start);
            test("prepend", degrees, axis, pivot, null);
            test("prepend", degrees, axis, pivot, start);
        }

        private function test(method:String, degrees:Number, axis:Vector3D, pivot:Vector3D, start:Vector.<Number>):void {
            var matrix:Matrix3D = start == null ? new Matrix3D() : new Matrix3D(start);
            if (method == "append") {
                matrix.appendRotation(degrees, axis, pivot);
            } else {
                matrix.prependRotation(degrees, axis, pivot);
            }
            trace("  " + method + (pivot == null ? "" : " pivot") + (start == null ? "" : " start") + ": " + matrix.rawData);
        }

        private function random():Number {
            seed = (seed * 16807) % 2147483647;
            return seed / 2147483647;
        }

        private function uniform(min:Number, max:Number):Number {
            return min + (max - min) * random();
        }
    }
}
