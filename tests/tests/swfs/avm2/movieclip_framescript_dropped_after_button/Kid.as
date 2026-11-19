package {
    import flash.display.MovieClip;
    public class Kid extends MovieClip {
        public static var count:int = 0;
        public var id:int;
        public function Kid() { id = ++count; trace("  Kid#" + id + " ctor"); addFrameScript(0, f1); }
        private function f1():void { trace("  Kid#" + id + ": frame 1 script"); stop(); }
    }
}
