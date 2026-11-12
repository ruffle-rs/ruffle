package {
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Matrix3D;
    import flash.system.fscommand;

    public class Test extends Sprite {
        public function Test() {
            trace("=== MATRIX3D OWNERSHIP BEGIN ===");
            rejectedAssignment();
            for (var kind:int = 0; kind < 3; kind++) {
                release(kind);
            }
            trace("=== MATRIX3D OWNERSHIP END ===");
            fscommand("quit");
        }

        private function assign(label:String, target:Sprite, matrix:Matrix3D):void {
            try {
                target.transform.matrix3D = matrix;
                trace(label, "accepted");
            } catch (e:Error) {
                trace(label, e.constructor);
                trace(e.getStackTrace());
            }
        }

        private function rejectedAssignment():void {
            trace("=== rejected assignment ===");
            var a:Sprite = new Sprite();
            var b:Sprite = new Sprite();
            var ma:Matrix3D = new Matrix3D();
            var mb:Matrix3D = new Matrix3D();
            ma.appendTranslation(1, 2, 3);
            mb.appendTranslation(11, 22, 33);
            a.transform.matrix3D = ma;
            b.transform.matrix3D = mb;
            assign("same owner", a, ma);
            assign("second owner", b, ma);
            trace("target position after rejection", b.x, b.y, b.z);
            assign("target still owns previous matrix", a, mb);
        }

        private function release(kind:int):void {
            trace("=== release ===", ["replacement", "null", "2D matrix"][kind]);
            var a:Sprite = new Sprite();
            var b:Sprite = new Sprite();
            var old:Matrix3D = new Matrix3D();
            a.transform.matrix3D = old;
            switch (kind) {
                case 0:
                    a.transform.matrix3D = new Matrix3D();
                    break;
                case 1:
                    a.transform.matrix3D = null;
                    break;
                case 2:
                    a.transform.matrix = new Matrix(2, 0, 0, 3, 4, 5);
                    break;
            }
            assign("reuse released matrix", b, old);
            assign("new owner rejects sharing", a, old);
        }
    }
}
