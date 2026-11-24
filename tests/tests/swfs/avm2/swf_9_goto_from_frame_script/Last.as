package {
    import flash.display.MovieClip;

    public class Last extends MovieClip {
        public function Last() {
            addFrameScript(1, frame2);
            gotoAndStop(2);
        }

        public function frame2():void {
            trace("last frame2");
        }
    }
}
