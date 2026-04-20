package {
    import flash.display.MovieClip;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.MouseEvent;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        private var dragger:MovieClip;
        private var target:MovieClip;
        private var sawOver:Boolean = false;

        public function Test() {
            addEventListener(Event.ADDED_TO_STAGE, onAdded);
        }

        private function onAdded(_:Event):void {
            removeEventListener(Event.ADDED_TO_STAGE, onAdded);

            dragger = new MovieClip();
            dragger.graphics.beginFill(0xCC3333);
            dragger.graphics.drawRect(0, 0, 100, 100);
            dragger.graphics.endFill();
            dragger.x = 0;
            dragger.y = 0;
            dragger.buttonMode = true;
            addChild(dragger);

            target = new MovieClip();
            target.graphics.beginFill(0x33CC66);
            target.graphics.drawRect(0, 0, 120, 120);
            target.graphics.endFill();
            target.x = 200;
            target.y = 200;
            addChild(target);

            target.buttonMode = true;
            target.mouseChildren = false;

            target.addEventListener(MouseEvent.MOUSE_OVER, function(e:MouseEvent):void {
                trace("OVER buttonDown=" + e.buttonDown);
                sawOver = true;
            });
            target.addEventListener(MouseEvent.MOUSE_OUT, function(e:MouseEvent):void {
                trace("OUT buttonDown=" + e.buttonDown);
                if (sawOver) {
                    fscommand("quit");
                }
            });

            dragger.addEventListener(MouseEvent.MOUSE_DOWN, function(_:MouseEvent):void {
                dragger.startDrag(false);
                trace("startDrag");
            });
        }
    }
}
