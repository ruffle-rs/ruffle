package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        private var step:int = 0;
        private var ticks:int = 0;

        public function Test() {
            addFrameScript(0, frame1, 1, frame2, 2, frame3, 3, frame4);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function info(label:String):void {
            trace(label + ": frame " + currentFrame + ", isPlaying: " + isPlaying);
        }

        private function onEnterFrame(e:Event):void {
            ticks++;
            info("enterFrame " + ticks);

            if (ticks == 12) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }

        private function frame1():void {
            info("frame1 start");

            if (step == 0) {
                step = 1;
                play();
                info("after play()");
                nextFrame();
                info("after nextFrame()");
            }
        }

        private function frame2():void {
            info("frame2 start (step " + step + ")");

            if (step == 1) {
                step = 2;
                play();
                info("after play()");
            }
        }

        private function frame3():void {
            info("frame3 start (step " + step + ")");

            if (step == 2) {
                step = 3;
                prevFrame();
                info("after prevFrame()");
            }
        }

        private function frame4():void {
            info("frame4 start (step " + step + ")");
        }
    }
}
