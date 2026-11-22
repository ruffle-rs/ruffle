// nextScene and prevScene make the clip play: isPlaying becomes true once a
// queued scene change runs, or right away outside of a frame script.
package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        private var step:int = 0;
        private var ticks:int = 0;

        public function Test() {
            addFrameScript(0, frame1, 2, frame3);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function info(label:String):void {
            trace(label + ": " + currentScene.name + " frame " + currentFrame + ", isPlaying: " + isPlaying);
        }

        private function onEnterFrame(e:Event):void {
            ticks++;
            info("enterFrame " + ticks);

            if (step == 2 && ticks == 6) {
                step = 3;
                stop();
                info("enterFrame: after stop()");
                prevScene();
                info("enterFrame: after prevScene()");
            }

            if (ticks == 12) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }

        private function frame1():void {
            info("frame1 (step " + step + ")");

            if (step == 0) {
                step = 1;
                stop();
                info("frame1: after stop()");
                nextScene();
                info("frame1: after nextScene()");
            }
        }

        private function frame3():void {
            info("frame3 (step " + step + ")");

            if (step == 1) {
                step = 2;
            }
        }
    }
}
