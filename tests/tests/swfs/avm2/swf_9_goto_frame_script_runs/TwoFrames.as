package {
    import flash.display.MovieClip;

    public class TwoFrames extends MovieClip {
        public function TwoFrames() {
            addFrameScript(0, frame1, 1, frame2);
        }

        public function frame1():void {
            trace(name + " frame1, currentFrame = " + currentFrame);
            stop();
        }

        public function frame2():void {
            trace(name + " frame2, currentFrame = " + currentFrame);
        }
    }
}
