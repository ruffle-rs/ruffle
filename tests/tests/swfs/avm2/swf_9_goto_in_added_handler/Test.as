package {
    import flash.display.MovieClip;

    // Tests when the frame of a goto run during frame construction is
    // constructed in SWFv9, where a goto doesn't run a nested frame.
    //
    // The popup is sent to frame 2 from an enterFrame handler. Frame 2 places
    // "content" and "sib", and the popup sends "content" to frame 3 from an
    // ADDED handler, while "content" is being constructed. Frame 3 of
    // "content" places "comp", which its frame script then uses. "later" is
    // placed on the main timeline in the same frame, after the popup. The
    // test ends when the popup stops on its last frame.
    //
    // This is what Twin Shot does to set the text of its level complete
    // popup (#25321).
    public class Test extends MovieClip {
        public var popup:Popup;
        public var later:Named;

        private var enterFrames:int = 0;

        public function Test() {
            addFrameScript(3, frame4);

            addEventListener("enterFrame", onEnterFrame);
            addEventListener("frameConstructed", onFrameConstructed);
            addEventListener("exitFrame", onExitFrame);
        }

        private function onEnterFrame(e:*):void {
            trace("-- enterFrame " + enterFrames + " " + state());

            if (enterFrames == 1) {
                trace("popup.gotoAndPlay(2)");
                popup.gotoAndPlay(2);
                trace("after popup.gotoAndPlay(2): " + state());
            }

            enterFrames++;
        }

        private function onFrameConstructed(e:*):void {
            trace("-- frameConstructed " + state());
        }

        private function onExitFrame(e:*):void {
            trace("-- exitFrame " + state());

            if (popup.currentFrame == popup.totalFrames) {
                removeEventListener("enterFrame", onEnterFrame);
                removeEventListener("frameConstructed", onFrameConstructed);
                removeEventListener("exitFrame", onExitFrame);
                trace("Done");
            }
        }

        private function state():String {
            var content:Content = popup ? popup.content : null;
            var comp:Comp = content ? content.comp : null;

            return "content=" + (content ? content.currentFrame : null) +
                " comp=" + (comp ? comp.text : null);
        }

        public function frame4():void {
            stop();
        }
    }
}
