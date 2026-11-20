package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class ProbeClip extends MovieClip {
        public var child:MovieClip;
        public var label:String = "initializing";

        public function ProbeClip() {
            addFrameScript(0, frame1);
        }

        private function frame1():void {
            stop();
            trace(label + " frame script 1");
            if (label == "sender") {
                dispatchEvent(new Event("probe", true));
            }
        }
    }
}
