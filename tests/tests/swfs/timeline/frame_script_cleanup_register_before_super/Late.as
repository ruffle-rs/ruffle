package {
    import flash.display.MovieClip;

    public class Late extends MovieClip {
        public function Late() {
            trace("Late constructor, currentFrame " + currentFrame);
            super();
            addFrameScript(0, frame1);
            trace("Late after super(), currentFrame " + currentFrame);
        }

        private function frame1():void {
            trace("Late frame1 (frame " + Main.frames + ")");
        }
    }
}
