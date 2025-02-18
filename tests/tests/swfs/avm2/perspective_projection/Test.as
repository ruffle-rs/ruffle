package {
    import flash.display.*;
    import flash.geom.*;

    [SWF(width="300", height="200")]
    public class Test extends Sprite {
        public function Test() {
            TestDefault();
            trace("");

            TestTransformDefaultValues();
            trace("");

            TestFOVtoFL();
            trace("");

            TestFLtoFOV();
            trace("");

            TestSetters();
            trace("");

            TestToMatrix3D();
            trace("");

            TestTransform();
            trace("");

            //// FIXME: DisplayObject.transform.perspectiveProjection setters should update the associated DO. Unimplemented now.
            // TestTransformUpdate();
            // trace("");
        }

        private function TestDefault(): void {
            trace("// Default");
            printProps(new PerspectiveProjection());
        }

        private function TestTransformDefaultValues(): void {
            trace("// Default values (stage)");
            printProps(stage.transform.perspectiveProjection);
            trace("// Default values (root)");
            printProps(root.transform.perspectiveProjection);
            trace("// Default values (object)");
            printProps((new Sprite()).transform.perspectiveProjection);

        }

        private function TestFOVtoFL(): void {
            for (var i: int = 1; i < 180; i++) {
                var pp: PerspectiveProjection = new PerspectiveProjection();
                pp.fieldOfView = i;
                var fl: Number = pp.focalLength;
                trace("FOV to FL", i, roundN(fl, 100)); // FIXME: Large numerical errors
            }
        }

        private function TestFLtoFOV(): void {
            for (var i: int = 1; i < 1000; i++) {
                var pp: PerspectiveProjection = new PerspectiveProjection();
                pp.focalLength = i;
                var fl: Number = pp.fieldOfView;
                trace("FL to FOV", i, roundN(fl, 100000000000));
            }
        }

        private function TestSetters(): void {
            var pp: PerspectiveProjection = new PerspectiveProjection();

            trace("// Default");
            printProps(pp);

            trace("// FOV: 1");
            pp.fieldOfView = 1;
            printProps(pp);

            trace("// FOV: 100");
            pp.fieldOfView = 100;
            printProps(pp);

            trace("// FOV: 179");
            pp.fieldOfView = 179;
            printProps(pp);

            trace("// FL: 1");
            pp.focalLength = 1;
            printProps(pp);

            trace("// FL: 10");
            pp.focalLength = 10;
            printProps(pp);

            trace("// FL: 10000");
            pp.focalLength = 10000;
            printProps(pp);

            trace("// center: (0, 0)");
            pp.projectionCenter = new Point(0, 0);
            printProps(pp);

            trace("// center: (100, -100)");
            pp.projectionCenter = new Point(100, -100);
            printProps(pp);

            trace("// center: (1000, 10)");
            pp.projectionCenter = new Point(1000, 10);
            printProps(pp);
        }

        private function TestToMatrix3D(): void {
            var precision: Number = 100;
            var pp: PerspectiveProjection = new PerspectiveProjection();

            trace("// toMatrix3D(default)");
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FOV: 1)");
            pp.fieldOfView = 1;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FOV: 100)");
            pp.fieldOfView = 100;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FOV: 179)");
            pp.fieldOfView = 179;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FL: 1)");
            pp.focalLength = 1;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FL: 10)");
            pp.focalLength = 10;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));

            trace("// toMatrix3D(FL: 10000)");
            pp.focalLength = 10000;
            trace(roundVecN(pp.toMatrix3D().rawData, precision));
        }

        private function TestTransform(): void {
            var s: Sprite = new Sprite();

            var pp: PerspectiveProjection = new PerspectiveProjection();
            printProps(pp);

            trace("// Set non-null to transform.perspectiveProjection");
            s.transform.perspectiveProjection = pp;
            printProps(pp);
            printProps(s.transform.perspectiveProjection);

            trace("// Set null to transform.perspectiveProjection");
            s.transform.perspectiveProjection = null;
            printProps(pp);
            printProps(s.transform.perspectiveProjection);
        }

        private function TestTransformUpdate(): void {
            var s: Sprite = new Sprite();

            trace("// Set default PerspectiveProjection to transform");
            s.transform.perspectiveProjection = new PerspectiveProjection();
            printProps(s.transform.perspectiveProjection);

            trace("// Set FOV = 100");
            s.transform.perspectiveProjection.fieldOfView = 100;
            printProps(s.transform.perspectiveProjection);

            trace("// Set FL = 10000");
            s.transform.perspectiveProjection.focalLength = 10000;
            printProps(s.transform.perspectiveProjection);

            trace("// Set center = (10, 10)");
            s.transform.perspectiveProjection.projectionCenter = new Point(10, 10);
            printProps(s.transform.perspectiveProjection);
        }

        private function printProps(pp: PerspectiveProjection): void {
            trace("  perspectiveProjection = " + pp);
            if (pp) {
                trace("  perspectiveProjection.fieldOfView = " + roundN(pp.fieldOfView, 100000000000));
                trace("  perspectiveProjection.focalLength = " + roundN(pp.focalLength, 100)); // FIXME: Large numerical errors
                trace("  perspectiveProjection.projectionCenter = " + pp.projectionCenter);
            }
        }

        private function roundN(n: Number, precision: Number): Number {
            return Math.round(n * precision) / precision;
        }

        private function roundVecN(v: Vector.<Number>, precision: Number): Vector.<Number> {
            return v.map(
                function (n: Number, _, _): Number {return roundN(n, precision);}
                );
        }
    }
}
