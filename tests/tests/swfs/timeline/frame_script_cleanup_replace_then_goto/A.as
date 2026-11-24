package {
    import flash.display.MovieClip;

    public class A extends MovieClip {
        public function A() {
            trace("A constructor");
            addFrameScript(0, frame1);
        }

        public function replaceFrameScript():void {
            trace("A replacing its frame script");
            addFrameScript(0, frame1b);
        }

        private function frame1():void {
            trace("A frame1");
        }

        private function frame1b():void {
            trace("A frame1b");
        }
    }
}
