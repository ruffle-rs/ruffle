package {
    import flash.display.MovieClip;

    public class A extends MovieClip {
        public function A() {
            trace(name + " constructor");
            addFrameScript(0, frame1);
        }

        public function removeFrameScript():void {
            trace(name + " removing its frame script");
            addFrameScript(0, null);
        }

        public function replaceFrameScript():void {
            trace(name + " replacing its frame script");
            addFrameScript(0, frame1b);
        }

        private function frame1():void {
            trace(name + " frame1");
        }

        private function frame1b():void {
            trace(name + " frame1b");
        }
    }
}
