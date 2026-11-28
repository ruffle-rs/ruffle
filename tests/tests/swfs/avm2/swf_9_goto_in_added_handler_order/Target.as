package {
    import flash.display.MovieClip;

    public class Target extends MovieClip {
        public var kid:Kid;

        public function Target() {
            addFrameScript(1, frame2);
            stop();
        }

        public function frame2():void {
            trace(name + " frame2, " + state());
        }

        public function state():String {
            return "currentFrame = " + currentFrame + ", kid = " + kid;
        }
    }
}
