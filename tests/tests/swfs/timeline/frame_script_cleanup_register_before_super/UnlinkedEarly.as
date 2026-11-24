package {
    import flash.display.MovieClip;

    public class UnlinkedEarly extends MovieClip {
        public function UnlinkedEarly() {
            trace("UnlinkedEarly constructor, currentFrame " + currentFrame);
            addFrameScript(0, frame1);
            super();
            trace("UnlinkedEarly after super(), currentFrame " + currentFrame);
        }

        private function frame1():void {
            trace("UnlinkedEarly frame1 (frame " + Main.frames + ")");
        }
    }
}
