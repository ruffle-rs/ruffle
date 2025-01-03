package  {
    import flash.display.*;
    import flash.text.*;
    import flash.events.*;

    [SWF(width="800", height="400")]
    public class Test extends MovieClip {
        private var text: TextField;

        public function Test() {
            var a = newSprite()
            a.x = 0;
            a.y = 0;
            a.width = 200;
            a.height = 400;
            a.addEventListener(MouseEvent.MOUSE_WHEEL, consumeWheel1);
            addChild(a);

            var b = newSprite()
            b.x = 400;
            b.y = 0;
            b.width = 200;
            b.height = 400;
            b.addEventListener(MouseEvent.MOUSE_WHEEL, consumeWheel2);
            addChild(b);

            var c = new TextField();
            c.mouseWheelEnabled = true;
            c.border = true;
            c.x = 200;
            c.y = 0;
            c.width = 200;
            c.height = 400;
            c.multiline = true;
            for (var i = 0; i < 100; ++ i) {
                c.text += "line\n";
            }
            addChild(c);
            text = c;

            trace("Loaded!");
        }

        function consumeWheel1(event: MouseEvent) {
            trace("Wheel consumed 1, vscroll: " + text.scrollV);
        }

        function consumeWheel2(event: MouseEvent) {
            trace("Wheel consumed 2, vscroll: " + text.scrollV);
            event.preventDefault();
        }

        function handleScroll(event: Event) {
            trace("Scrolled");
        }

        private function newSprite(): Sprite {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF00FF);
            s.graphics.drawRect(0, 0, 200, 400);
            s.graphics.endFill();
            return s;
        }
    }
}
