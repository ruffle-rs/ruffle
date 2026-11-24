package {
    import flash.display.MovieClip;

    // Tests in which order the frames of gotos from a frame script are
    // constructed in SWFv9, where a goto doesn't run a nested frame.
    //
    // All barrels are sent to frame 3, which places "inner". The frame script
    // of the main timeline sends "dkD", "dkB" and "dkA" there, which is not
    // display list order, and then "off", which was removed from the display
    // list before the frame scripts ran. The constructor of the inner clip of
    // "dkD" sends "dkC" to frame 3 as well. The frame 3 script of each barrel
    // traces its inner clip, except for "off".
    public class Test extends MovieClip {
        public var dkA:Barrel;
        public var dkB:Barrel;
        public var off:Barrel;
        public var dkC:Barrel;
        public var dkD:Barrel;

        private var held:Barrel;
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

            if (tick == 0) {
                held = off;
                removeChild(held);
            }
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame " + tick + " " + state());

            if (tick == 2) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }

            tick++;
        }

        private function state():String {
            return "A=" + (dkA ? dkA.inner : null) + " B=" + (dkB ? dkB.inner : null) +
                " C=" + (dkC ? dkC.inner : null) + " D=" + (dkD ? dkD.inner : null) +
                " held=" + (held ? held.inner + " (frame " + held.currentFrame + ")" : null);
        }

        public function frame1():void {
            trace("Test frame1");

            dkD.gotoAndStop(3);
            dkB.gotoAndStop(3);
            dkA.gotoAndStop(3);
            held.gotoAndStop(3);
            trace("Test frame1 end " + state());
            stop();
        }
    }
}
