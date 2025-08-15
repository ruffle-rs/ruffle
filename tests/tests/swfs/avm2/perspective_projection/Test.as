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

            TestTransformUpdate();
            trace("");

            TestTransformUpdateSetterSync();
            trace("");

            TestTransformUpdateGetterSync();
            trace("");

            TestTransformUpdateDoubleRef();
            trace("");

            TestTransformStage();
            trace("");

            TestTransformRoot();
            trace("");
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
                trace("FOV to FL", i, pp.focalLength);
            }
        }

        private function TestFLtoFOV(): void {
            for (var i: int = 1; i < 1000; i++) {
                var pp: PerspectiveProjection = new PerspectiveProjection();
                pp.focalLength = i;
                trace("FL to FOV", i, pp.fieldOfView);
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
            var pp: PerspectiveProjection = new PerspectiveProjection();

            trace("// toMatrix3D(default)");
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FOV: 1)");
            pp.fieldOfView = 1;
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FOV: 100)");
            pp.fieldOfView = 100;
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FOV: 179)");
            pp.fieldOfView = 179;
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FL: 1)");
            pp.focalLength = 1;
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FL: 10)");
            pp.focalLength = 10;
            trace(pp.toMatrix3D().rawData);

            trace("// toMatrix3D(FL: 10000)");
            pp.focalLength = 10000;
            trace(pp.toMatrix3D().rawData);
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

        private function TestTransformUpdateSetterSync(): void {
            var s: Sprite = new Sprite();
            var pp: PerspectiveProjection = new PerspectiveProjection();
            s.transform.perspectiveProjection = pp;

            trace("// init");
            printProps(s.transform.perspectiveProjection);
            printProps(pp);

            trace("// Set pp.FOV = 100");
            pp.fieldOfView = 100;
            printProps(s.transform.perspectiveProjection);
            printProps(pp);

            trace("// Set transform.FOV = 150");
            s.transform.perspectiveProjection.fieldOfView = 150;
            printProps(s.transform.perspectiveProjection);
            printProps(pp);
        }

        private function TestTransformUpdateGetterSync(): void {
            var s: Sprite = new Sprite();
            s.transform.perspectiveProjection = new PerspectiveProjection();
            var pp: PerspectiveProjection = s.transform.perspectiveProjection;

            trace("// init");
            printProps(s.transform.perspectiveProjection);
            printProps(pp);

            trace("// Set pp.FOV = 100");
            pp.fieldOfView = 100;
            printProps(s.transform.perspectiveProjection);
            printProps(pp);

            trace("// Set transform.FOV = 150");
            s.transform.perspectiveProjection.fieldOfView = 150;
            printProps(s.transform.perspectiveProjection);
            printProps(pp);
        }

        private function TestTransformUpdateDoubleRef(): void {
            var s: Sprite = new Sprite();
            s.transform.perspectiveProjection = new PerspectiveProjection();
            var pp1: PerspectiveProjection = s.transform.perspectiveProjection;
            var pp2: PerspectiveProjection = s.transform.perspectiveProjection;

            trace("// init");
            printProps(s.transform.perspectiveProjection);
            printProps(pp1);
            printProps(pp2);

            trace("// Set pp1.FOV = 100");
            pp1.fieldOfView = 100;
            printProps(s.transform.perspectiveProjection);
            printProps(pp1);
            printProps(pp2);

            trace("// Set pp2.center = (10, 10)");
            pp2.projectionCenter = new Point(10, 10);
            printProps(s.transform.perspectiveProjection);
            printProps(pp1);
            printProps(pp2);
        }

        private function TestTransformStage(): void {
            var pp: PerspectiveProjection = new PerspectiveProjection();
            pp.fieldOfView = 123;

            trace("// stage default");
            printProps(stage.transform.perspectiveProjection);

            trace("// set null to stage");
            stage.transform.perspectiveProjection = null;
            printProps(stage.transform.perspectiveProjection);

            trace("// set default to stage");
            stage.transform.perspectiveProjection = pp;
            printProps(stage.transform.perspectiveProjection);

            trace("// set null to stage");
            stage.transform.perspectiveProjection = null;
            printProps(stage.transform.perspectiveProjection);
        }

        private function TestTransformRoot(): void {
            var pp: PerspectiveProjection = new PerspectiveProjection();
            pp.fieldOfView = 123;

            trace("// root default");
            printProps(root.transform.perspectiveProjection);

            trace("// set null to root");
            root.transform.perspectiveProjection = null;
            printProps(root.transform.perspectiveProjection);

            trace("// set pp to root");
            root.transform.perspectiveProjection = pp;
            printProps(root.transform.perspectiveProjection);

            trace("// set null to root");
            root.transform.perspectiveProjection = null;
            printProps(root.transform.perspectiveProjection);
        }

        private function printProps(pp: PerspectiveProjection): void {
            trace("  perspectiveProjection = " + pp);
            if (pp) {
                trace("  perspectiveProjection.fieldOfView = " + pp.fieldOfView);
                trace("  perspectiveProjection.focalLength = " + pp.focalLength);
                trace("  perspectiveProjection.projectionCenter = " + pp.projectionCenter);
            }
        }
    }
}
