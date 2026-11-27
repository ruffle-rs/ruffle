package {
    import flash.display.MovieClip;
    public class Kid extends MovieClip {
        public static var count:int = 0;
        private static var ran:Array = [];
        public var id:int;
        public function Kid() { id = ++count; trace("  Kid#" + id + " ctor"); addFrameScript(0, f1); }
        private function f1():void { ran.push(id); stop(); }

        // The frame scripts of the kids in the up and over states of a button
        // don't run in the same order in every run of Flash Player, so the
        // kids whose frame script ran are traced in ID order at the next trace.
        public static function traceRan():void {
            if (ran.length == 0) return;
            ran.sort(Array.NUMERIC);
            trace("  Kid#" + ran.join(", Kid#") + ": frame 1 script");
            ran = [];
        }
    }
}
