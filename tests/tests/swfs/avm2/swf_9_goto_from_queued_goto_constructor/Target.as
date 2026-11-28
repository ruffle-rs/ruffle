package {
    import flash.display.MovieClip;

    public class Target extends MovieClip {
        public var inner:Inner;

        public function Target() {
            addFrameScript(0, frame1, 2, frame3);
        }

        public function frame1():void {
            trace(name + " frame1");
            stop();
        }

        public function frame3():void {
            trace(name + " frame3, inner = " + inner);
        }
    }
}
