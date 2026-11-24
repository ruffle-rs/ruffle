package {
    import flash.display.MovieClip;

    public class D extends MovieClip {
        public function D() {
            addFrameScript(0, frame1);
        }

        private function frame1():void {
            trace("d frame1 (frame " + Main.frames + ")");
        }
    }
}
