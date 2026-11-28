package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Adder extends MovieClip {
        public var added:Named;

        public function Adder() {
            addFrameScript(1, frame2);
            addEventListener(Event.ADDED, onAdded);
        }

        private function onAdded(e:Event):void {
            if (e.target.parent != this) {
                return;
            }

            var test:Test = Test(parent);

            trace("adder ADDED " + e.target.name);
            test.targetBefore.gotoAndStop(2);
            test.targetAfter.gotoAndStop(2);
            trace("after targetBefore/targetAfter.gotoAndStop(2): targetBefore " + test.targetBefore.state() +
                ", targetAfter " + test.targetAfter.state());
        }

        public function frame2():void {
            trace("adder frame2");
            stop();
        }
    }
}
