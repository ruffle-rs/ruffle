package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Main extends MovieClip {
        public var a1:A;
        public var a2:A;

        private var frames:int = 1;

        public function Main() {
            trace("Main constructor");
            addFrameScript(0, frame1);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
            addEventListener(Event.EXIT_FRAME, onExitFrame);
        }

        private function frame1():void {
            trace("Main frame1 start");
            a1.replaceFrameScript();
            a2.removeFrameScript();
            a2.replaceFrameScript();
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
