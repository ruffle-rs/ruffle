package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Main extends MovieClip {
        public var a:A;
        public var b:B;

        private var frames:int = 1;

        public function Main() {
            trace("Main constructor");
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
            addEventListener(Event.EXIT_FRAME, onExitFrame);
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
