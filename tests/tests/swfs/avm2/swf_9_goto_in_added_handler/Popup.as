package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Popup extends MovieClip {
        public var content:Content;
        public var sib:Named;

        public function Popup() {
            addFrameScript(0, frame1, 3, frame4);
            addEventListener(Event.ADDED, onAdded);
        }

        private function onAdded(e:Event):void {
            if (e.target.parent == this && e.target is Content) {
                trace("Popup ADDED " + e.target.name + ", gotoAndStop(3)");

                MovieClip(e.target).gotoAndStop(3);
                trace("after gotoAndStop(3): content=" + content + " comp=" + e.target.comp);
            }
        }

        public function frame1():void {
            stop();
        }

        public function frame4():void {
            stop();
        }
    }
}
