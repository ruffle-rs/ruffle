package {
    import flash.display.MovieClip;

    public class Early extends MovieClip {
        public function Early() {
            trace("Early constructor, currentFrame " + currentFrame);
            addFrameScript(0, frame1);
            super();
            trace("Early after super(), currentFrame " + currentFrame);
        }

        private function frame1():void {
            trace("Early frame1 (frame " + Main.frames + ")");
        }
    }
}
