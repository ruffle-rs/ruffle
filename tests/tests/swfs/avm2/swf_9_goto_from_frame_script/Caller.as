package {
    import flash.display.MovieClip;

    public class Caller extends MovieClip {
        public function Caller() {
            addFrameScript(0, frame1);
        }

        public function frame1():void {
            var test:Test = Test(parent);

            trace("caller frame1");

            test.dkE.gotoAndStop(3);
            trace("after dkE.gotoAndStop(3): E=" + test.dkE.inner);

            test.dkF.gotoAndStop(3);
            trace("after dkF.gotoAndStop(3): F=" + test.dkF.inner);

            test.last.gotoAndStop(9999);
            trace("after last.gotoAndStop(9999): last.currentFrame=" + test.last.currentFrame);

            stop();
        }
    }
}
