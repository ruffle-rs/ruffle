package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        public var clip:MovieClip;
        private var boxes:Array = [];
        private var ticks:int = 0;

        public function Test() {
            addFrameScript(0, frame1);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function frame1():void {
            trace("root frame 1 script");
            boxes.push(new Box("beforeGoto"));
            trace("clip.gotoAndStop(2)");
            clip.gotoAndStop(2);
            boxes.push(new Box("afterGoto"));
            trace("root frame 1 script end");
        }

        private function onEnterFrame(e:Event):void {
            ticks++;
            var parts:Array = [];
            for each (var box:Box in boxes) {
                parts.push(box.label + "=" + box.currentFrame);
            }
            trace("tick " + ticks + ": " + parts.join(", "));
            if (ticks == 4) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }
    }
}
