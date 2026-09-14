package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public var counter:int;

        public function Test() {
            super();

            this.counter = 0;

            addFrameScript(0, frame1, 1, frame2, 2, frame3);
        }

        public function frame1():void {
            // Call `play();` so that `isPlaying` shows `true`
            play();
            trace("frame1, isPlaying: " + this.isPlaying);
        }

        public function frame2():void {
            counter ++;
            trace("frame2, isPlaying: " + this.isPlaying);

            if (counter < 3) {
                gotoAndStop(2);
                trace("// gotoAndStop(2);");
                trace("isPlaying: " + this.isPlaying);
            } else {
                trace("isPlaying: " + this.isPlaying);
            }
        }

        public function frame3():void {
            trace("frame3, isPlaying: " + this.isPlaying);
            stop();
        }
    }
}
