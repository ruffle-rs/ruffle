package {
    import flash.display.MovieClip;

    public class B extends MovieClip {
        public function B() {
            trace("B constructor");
            addFrameScript(1, frame2);
        }

        private function frame2():void {
            trace("B frame2 (frame " + Main.frames + ")");
        }
    }
}
