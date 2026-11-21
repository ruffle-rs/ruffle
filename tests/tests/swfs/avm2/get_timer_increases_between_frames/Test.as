package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.utils.getTimer;

    // Tests that getTimer() increases between frames, by tracing for each of
    // 30 frames whether it is greater than in the previous one.
    public class Test extends Sprite {
        private var frames:int = 0;
        private var last:int = -1;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function onEnterFrame(event:Event):void {
            var time:int = getTimer();
            frames++;
            trace("frame " + frames + ": increased " + (time > last));
            last = time;

            if (frames == 30) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("Done");
            }
        }
    }
}
