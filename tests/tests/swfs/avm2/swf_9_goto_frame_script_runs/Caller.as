package {
    import flash.display.MovieClip;

    public class Caller extends MovieClip {
        public function Caller() {
            addFrameScript(1, frame2, 2, frame3, 3, frame4, 4, frame5, 5, frame6, 6, frame7);
        }

        private function get test():Test {
            return Test(parent);
        }

        public function frame2():void {
            trace("caller frame2");
            test.waiting.gotoAndStop(4);
            test.notWaiting.gotoAndStop(4);
            trace("after waiting/notWaiting.gotoAndStop(4): waiting.currentFrame = " +
                test.waiting.currentFrame + ", notWaiting.currentFrame = " + test.notWaiting.currentFrame);
        }

        public function frame3():void {
            trace("caller frame3");
            test.noOldScript.gotoAndStop(3);
            trace("after noOldScript.gotoAndStop(3): noOldScript.currentFrame = " +
                test.noOldScript.currentFrame + ", noOldScript.kid = " + test.noOldScript.kid);
        }

        public function frame4():void {
            trace("caller frame4");
            test.late.gotoAndStop(9999);
            trace("after late.gotoAndStop(9999): late.currentFrame = " + test.late.currentFrame);
        }

        public function frame5():void {
            trace("caller frame5");
            test.early.gotoAndStop(9999);
            trace("after early.gotoAndStop(9999): early.currentFrame = " + test.early.currentFrame);
        }

        public function frame6():void {
            trace("caller frame6");
            test.control.gotoAndStop(2);
            trace("after control.gotoAndStop(2): control.currentFrame = " + test.control.currentFrame);
        }

        public function frame7():void {
            trace("caller frame7");
            test.controlAfterTail.gotoAndStop(2);
            trace("after controlAfterTail.gotoAndStop(2): controlAfterTail.currentFrame = " +
                test.controlAfterTail.currentFrame);

            stop();
        }
    }
}
