package {
    import flash.display.MovieClip;

    // Tests when the frame of a goto on another clip is constructed and when
    // its frame script runs in SWFv9, where a goto doesn't run a nested frame:
    // - dkA, dkB and dkD are sent to frame 3 from a frame script; dkA and dkD
    //   have a frame script there, dkB doesn't, and dkD plays on.
    // - dkC is sent to frame 3 from an enterFrame handler.
    // - dkE and dkF are sent to frame 3 from the frame script of "caller",
    //   which comes after dkE and before dkF on the display list.
    // - "last" is sent to a frame past its last one, which it's already on.
    // Their frame 3 places a child named "inner", whose constructor adds a
    // frame script to its first frame.
    public class Test extends MovieClip {
        public var dkA:BarrelA;
        public var dkB:BarrelB;
        public var dkC:BarrelA;
        public var dkD:BarrelA;
        public var dkE:BarrelA;
        public var caller:Caller;
        public var dkF:BarrelA;
        public var last:Last;

        private var enterFrames:int = 0;

        public function Test() {
            addFrameScript(0, frame1, 1, frame2);

            addEventListener("enterFrame", onEnterFrame);
            addEventListener("frameConstructed", onFrameConstructed);
            addEventListener("exitFrame", onExitFrame);
        }

        private function inners():String {
            return "A=" + (dkA ? dkA.inner : null) + " B=" + (dkB ? dkB.inner : null) +
                " C=" + (dkC ? dkC.inner : null) + " D=" + (dkD ? dkD.inner : null);
        }

        private function onEnterFrame(e:*):void {
            trace("-- enterFrame " + enterFrames);

            if (enterFrames == 0) {
                dkC.gotoAndStop(3);
                trace("after dkC.gotoAndStop(3): " + inners());
            }

            enterFrames++;
        }

        private function onFrameConstructed(e:*):void {
            trace("-- frameConstructed " + inners());
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame " + inners());

            if (enterFrames == 1) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }
        }

        public function frame1():void {
            trace("Test frame1");

            dkA.gotoAndStop(3);
            trace("after dkA.gotoAndStop(3): " + inners());

            dkB.gotoAndStop(3);
            trace("after dkB.gotoAndStop(3): " + inners());

            dkD.gotoAndPlay(3);
            trace("after dkD.gotoAndPlay(3): " + inners());
        }

        public function frame2():void {
            trace("Test frame2 " + inners());
            stop();
        }
    }
}
