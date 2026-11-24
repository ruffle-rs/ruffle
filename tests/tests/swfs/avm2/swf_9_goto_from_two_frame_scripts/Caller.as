package {
    import flash.display.MovieClip;

    public class Caller extends MovieClip {
        public function Caller() {
            addFrameScript(0, frame1);
        }

        public function frame1():void {
            trace("caller frame1: a.gotoAndStop(3)");
            Test(parent).a.gotoAndStop(3);
            stop();
        }
    }
}
