package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Main extends MovieClip {
        public var c:MovieClip;
        public var d:D;

        public static var frames:int = 1;

        public function Main() {
            trace("Main constructor");
            addFrameScript(0, frame1);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
            addEventListener(Event.EXIT_FRAME, onExitFrame);
        }

        private function frame1():void {
            trace("Main frame1 start");
            c.addChild(new Early());
            c.addChild(new Late());
            c.addChild(new UnlinkedEarly());
            c.addChild(new UnlinkedLate());
            trace("Main frame1 end");
        }

        private function onEnterFrame(e:Event):void {
            frames++;
            trace("enterFrame " + frames);
        }

        private function onExitFrame(e:Event):void {
            trace("exitFrame " + frames);
            if (frames >= 3) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                removeEventListener(Event.EXIT_FRAME, onExitFrame);
            }
        }
    }
}
