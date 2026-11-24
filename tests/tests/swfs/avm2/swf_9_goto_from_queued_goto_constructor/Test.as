package {
    import flash.display.MovieClip;

    // Tests when the frame of a goto run by a constructor during the frame
    // script phase is constructed in SWFv9, where a goto doesn't run a nested
    // frame.
    //
    // The frame script of "jumper" sends it to frame 2, which is queued until
    // that frame script returns. Frame 2 places "starter", whose constructor
    // sends "target", which comes after "jumper" on the display list, to
    // frame 3. Frame 3 of "target" places a child named "inner", which its
    // frame script uses.
    public class Test extends MovieClip {
        public var jumper:Jumper;
        public var target:Target;

        private var tick:int = 0;

        public function Test() {
            addFrameScript(0, frame1);

            addEventListener("enterFrame", onEnterFrame);
            addEventListener("frameConstructed", onFrameConstructed);
            addEventListener("exitFrame", onExitFrame);
        }

        private function onEnterFrame(e:*):void {
            trace("-- enterFrame " + tick + " " + state());
        }

        private function onFrameConstructed(e:*):void {
            trace("-- frameConstructed " + tick + " " + state());
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame " + tick + " " + state());

            if (tick == 1) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }

            tick++;
        }

        private function state():String {
            return "target.inner=" + (target ? target.inner : null);
        }

        public function frame1():void {
            stop();
        }
    }
}
