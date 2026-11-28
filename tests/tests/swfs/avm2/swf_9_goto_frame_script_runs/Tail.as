package {
    import flash.display.MovieClip;

    public class Tail extends MovieClip {
        public function Tail() {
            for (var i:int = 0; i < totalFrames; i++) {
                addFrameScript(i, frameScript);
            }
        }

        public function frameScript():void {
            trace("tail frame" + currentFrame);

            if (currentFrame == totalFrames) {
                stop();
            }
        }
    }
}
