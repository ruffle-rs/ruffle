package {
    import flash.display.MovieClip;
    public class Box extends MovieClip {
        public var label:String;
        public function Box(label:String, stopNow:Boolean) {
            this.label = label;
            addFrameScript(0, frame1);
            if (stopNow) stop();
        }
        private function frame1():void {
            Kid.traceRan();
            trace(label + ": frame 1 script");
            stop();
        }
    }
}
