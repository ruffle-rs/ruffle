package {
    import flash.display.MovieClip;

    // Tests when the frame of a goto run during frame construction is
    // constructed in SWFv9, when the goto is on an already constructed
    // sibling of the clip whose frame is being constructed.
    //
    // "adder" plays into frame 2, which places "added". From its ADDED
    // handler, "adder" sends "targetBefore" and "targetAfter", which come
    // before and after it on the display list, to frame 2. Their frame 2
    // places "kid", whose constructor traces, and has a frame script. "other"
    // and the main timeline's "later" are also constructed in this frame,
    // after "targetAfter".
    public class Test extends MovieClip {
        public var targetBefore:Target;
        public var adder:Adder;
        public var targetAfter:Target;
        public var other:Other;
        public var later:Named;

        private var enterFrames:int = 0;

        public function Test() {
            addFrameScript(1, frame2);

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

            if (enterFrames == 2) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }
        }

        public function frame2():void {
            trace("Test frame2");
            stop();
        }
    }
}
