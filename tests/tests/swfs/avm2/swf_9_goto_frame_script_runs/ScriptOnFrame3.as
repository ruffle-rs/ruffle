package {
    import flash.display.MovieClip;

    public class ScriptOnFrame3 extends MovieClip {
        public var kid:Kid;

        public function ScriptOnFrame3() {
            addFrameScript(2, frame3);
            stop();
        }

        public function frame3():void {
            trace(name + " frame3, currentFrame = " + currentFrame + ", kid = " + kid);
        }
    }
}
