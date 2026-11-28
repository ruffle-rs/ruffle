package {
    import flash.display.MovieClip;

    public class NotWaiting extends MovieClip {
        public function NotWaiting() {
            addFrameScript(0, frameScript, 3, frameScript);
        }

        public function frameScript():void {
            trace(name + " frame script, currentFrame = " + currentFrame);
        }
    }
}
