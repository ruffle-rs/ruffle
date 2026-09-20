package {
    import flash.display.MovieClip;

    // In SWFv9-10, gotoAndStop(undefined) inside a frame script is ignored,
    // while outside a frame script it goes to frame 1.
    public class Test extends MovieClip {
        public function Test() {
            super();
            stop();
            trace("construct: " + currentFrame);
            gotoAndStop(undefined);
            trace("after outside goto: " + currentFrame);
            this.addFrameScript(0, this.frame1, 3, this.frame4);
            gotoAndStop(4);
            trace("after goto 4: " + currentFrame);
        }

        public function frame1():void {
            trace("frame1");
            stop();
        }

        public function frame4():void {
            trace("frame4");
            gotoAndStop(undefined);
            trace("after undefined goto: " + currentFrame);
            stop();
        }
    }
}
