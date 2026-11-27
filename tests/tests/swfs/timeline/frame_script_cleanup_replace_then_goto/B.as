package {
    import flash.display.MovieClip;

    public class B extends MovieClip {
        public function B() {
            trace("B constructor");
            addFrameScript(0, frame1);
        }

        private function frame1():void {
            trace("B frame1 start");
            Main(parent).a.replaceFrameScript();
            trace("B gotoAndStop(1)");
            gotoAndStop(1);
            trace("B frame1 end");
        }
    }
}
