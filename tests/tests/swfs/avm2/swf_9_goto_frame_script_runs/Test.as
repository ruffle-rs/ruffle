package {
    import flash.display.MovieClip;

    // Tests when and how often the frame script of the new frame runs after a
    // goto from a frame script in SWFv9. On each of its frames 2-7, "caller"
    // sends one or two clips to another frame:
    // 2. "waiting" and "notWaiting" play and come after "caller". They're sent
    //    to frame 4, while "waiting" still has a frame script waiting on frame
    //    2 and "notWaiting" doesn't.
    // 3. "noOldScript" is stopped on frame 1, which has no frame script. It's
    //    sent to frame 3, which places "kid".
    // 4-5. "late" and "early", which come after and before "caller", are sent
    //    past their last frame (from frame 1 to frame 2).
    // 6-7. "control" and "controlAfterTail", which come before and after
    //    "tail", are sent to frame 2, which they do reach.
    // "late", "early", "control" and "controlAfterTail" stop in the frame script
    // of their frame 1. "tail" has a frame script on every frame and comes
    // after the other clips, so a frame script that runs before it runs when
    // the frame script phase reaches its clip, and one after it runs at the
    // end of the phase.
    public class Test extends MovieClip {
        public var early:TwoFrames;
        public var caller:Caller;
        public var late:TwoFrames;
        public var waiting:Waiting;
        public var notWaiting:NotWaiting;
        public var noOldScript:ScriptOnFrame3;
        public var control:TwoFrames;
        public var tail:Tail;
        public var controlAfterTail:TwoFrames;

        private var enterFrames:int = 0;

        public function Test() {
            addEventListener("enterFrame", onEnterFrame);
            addEventListener("frameConstructed", onFrameConstructed);
            addEventListener("exitFrame", onExitFrame);
        }

        private function onEnterFrame(e:*):void {
            trace("-- enterFrame " + enterFrames);
            enterFrames++;
        }

        private function onFrameConstructed(e:*):void {
            trace("-- frameConstructed");
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame");

            if (tail.currentFrame == tail.totalFrames) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }
        }
    }
}
