package {
    import flash.display.MovieClip;

    public class UnlinkedLate extends MovieClip {
        public function UnlinkedLate() {
            trace("UnlinkedLate constructor, currentFrame " + currentFrame);
            super();
            addFrameScript(0, frame1);
            trace("UnlinkedLate after super(), currentFrame " + currentFrame);
        }

        private function frame1():void {
            trace("UnlinkedLate frame1 (frame " + Main.frames + ")");
        }
    }
}
