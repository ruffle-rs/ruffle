package {
    import flash.display.*;
    import flash.events.Event;

    // hitTestPoint(x, y, true) on objects whose graphics are changed after they
    // were rendered. Each starts as a square from (0,0) to (100,100).
    public class Test extends Sprite {
        private var frame:int = 0;
        private var shape1:Shape = new Shape();
        private var shape2:Shape = new Shape();
        private var sprite1:Sprite = new Sprite();
        private var sprite2:Sprite = new Sprite();

        public function Test() {
            var early:Shape = new Shape();
            square(early.graphics, 0);
            addChild(early);
            early.graphics.clear();
            square(early.graphics, 200);
            show("Shape never rendered, redrawn at x=200", early);

            for each (var o:DisplayObject in [shape1, shape2, sprite1, sprite2]) {
                square(o["graphics"], 0);
                addChild(o);
            }
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function onEnterFrame(e:Event):void {
            frame++;
            if (frame == 2) {
                // All of them have been rendered as squares at x=0 now.
                shape1.graphics.clear();
                square(shape1.graphics, 200);
                show("Shape rendered, then redrawn at x=200", shape1);
                shape2.graphics.clear();
                square(shape2.graphics, 200);
                shape2.x = 100;
                show("Shape rendered, then redrawn at x=200 and x=100", shape2);
                sprite1.graphics.clear();
                square(sprite1.graphics, 200);
                show("Sprite rendered, then redrawn at x=200", sprite1);
                sprite2.graphics.clear();
                square(sprite2.graphics, 200);
                sprite2.x = 100;
                show("Sprite rendered, then redrawn at x=200 and x=100", sprite2);
            }
            if (frame == 3) {
                show("next frame: Shape redrawn and moved", shape2);
                show("next frame: Sprite redrawn and moved", sprite2);
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }

        private function square(g:Graphics, x:Number):void {
            g.beginFill(0x808080);
            g.drawRect(x, 0, 100, 100);
            g.endFill();
        }

        private function show(label:String, o:DisplayObject):void {
            trace(label + ": shape(50,50)=" + o.hitTestPoint(50, 50, true) + " shape(250,50)=" + o.hitTestPoint(250, 50, true) +
                " shape(350,50)=" + o.hitTestPoint(350, 50, true) + " bounds(350,50)=" + o.hitTestPoint(350, 50, false));
        }
    }
}
