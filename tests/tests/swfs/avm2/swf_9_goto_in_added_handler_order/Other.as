package {
    import flash.display.MovieClip;

    public class Other extends MovieClip {
        public var otherKid:Named;

        public function Other() {
            addFrameScript(1, frame2);
        }

        public function frame2():void {
            trace("other frame2");
            stop();
        }
    }
}
