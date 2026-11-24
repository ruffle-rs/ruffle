package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public var a:Barrel;
        public var caller:Caller;

        private var tick:int = 0;

        public function Test() {
            addFrameScript(0, frame1);

            addEventListener("exitFrame", onExitFrame);
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame " + tick);

            if (tick == 1) {
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }

            tick++;
        }

        public function frame1():void {
            trace("Test frame1: a.gotoAndStop(2)");
            a.gotoAndStop(2);
            stop();
        }
    }
}
