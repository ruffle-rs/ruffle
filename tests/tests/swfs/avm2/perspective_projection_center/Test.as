package {
    import flash.display.Sprite;
    import flash.geom.PerspectiveProjection;
    import flash.geom.Point;

    public class Test extends Sprite {
        public function Test() {
            runCase("standalone", new PerspectiveProjection());

            var s:Sprite = new Sprite();
            addChild(s);
            s.transform.perspectiveProjection = new PerspectiveProjection();
            runCase("attached", s.transform.perspectiveProjection);

            testAssignment();

            trace("=== DONE ===");
        }

        private function show(label:String, p:PerspectiveProjection):void {
            try {
                var c:Point = p.projectionCenter;
                trace(label, c == null ? "null" : c.x + "," + c.y);
            } catch (e:Error) {
                trace(label, e.name, e.errorID);
            }
        }

        private function runCase(label:String, p:PerspectiveProjection):void {
            trace("=== " + label + " ===");

            var input:Point = new Point(10, 20);
            p.projectionCenter = input;
            show("after set", p);

            input.x = 100;
            input.y = 200;
            show("after mutating input", p);

            var first:Point = p.projectionCenter;
            var second:Point = p.projectionCenter;
            trace("getter identity", first === second);
            trace("input identity", first === input);

            first.x = 300;
            first.y = 400;
            show("after mutating getter result", p);

            try {
                p.projectionCenter = null;
                trace("set null: accepted");
            } catch (e:Error) {
                trace("set null:", e.name, e.errorID);
                trace("error string:", e.toString());
                trace("stack trace:", String(e.getStackTrace())
                    .replace(/\[[^\]]*\]/g, "")
                    .replace(/\r\n/g, "\n")
                    .replace(/^[ \t]+/gm, ""));
            }
            show("after null attempt", p);

            try {
                p.fieldOfView = 60;
                trace("set FOV after null: accepted");
            } catch (e:Error) {
                trace("set FOV after null:", e.name, e.errorID);
            }
            show("after FOV change", p);
        }

        private function testAssignment():void {
            trace("=== assignment ===");

            var input:Point = new Point(11, 22);
            var original:PerspectiveProjection = new PerspectiveProjection();
            original.projectionCenter = input;

            var s:Sprite = new Sprite();
            addChild(s);
            s.transform.perspectiveProjection = original;

            input.x = 111;
            show("original after input mutation", original);
            show("attached after input mutation",
                s.transform.perspectiveProjection);

            original.projectionCenter = new Point(33, 44);
            show("original after replacement", original);
            show("attached after original replacement",
                s.transform.perspectiveProjection);
        }
    }
}
