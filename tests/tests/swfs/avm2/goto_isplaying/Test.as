// Shows that isPlaying is a flag of its own, not whether the clip is playing:
// it starts out false, play(), gotoAndPlay(), nextScene() and prevScene() set
// it, only stop() clears it, and gotos that stop the clip leave it as it is.
// It never becomes true for a clip with a single frame.
package {
    import flash.display.MovieClip;
    import flash.events.Event;

    public class Test extends MovieClip {
        private var ticks:int = 0;

        public function Test() {
            info("constructor");

            var single:MovieClip = new MovieClip();
            trace("single frame clip: isPlaying " + single.isPlaying);
            single.play();
            trace("single frame clip after play(): isPlaying " + single.isPlaying);
            single.stop();
            trace("single frame clip after stop(): isPlaying " + single.isPlaying);

            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function info(label:String):void {
            trace(label + ": frame " + currentFrame + ", isPlaying: " + isPlaying);
        }

        private function onEnterFrame(e:Event):void {
            ticks++;
            info("tick " + ticks);

            switch (ticks) {
                case 1:
                    nextFrame();
                    info("nextFrame() without play()");
                    break;
                case 2:
                    play();
                    info("play()");
                    nextFrame();
                    info("nextFrame()");
                    break;
                case 3:
                    play();
                    prevFrame();
                    info("play(), prevFrame()");
                    break;
                case 4:
                    play();
                    gotoAndStop(1);
                    info("play(), gotoAndStop(1)");
                    break;
                case 5:
                    gotoAndPlay(2);
                    info("gotoAndPlay(2)");
                    break;
                case 6:
                    stop();
                    info("stop()");
                    gotoAndPlay(3);
                    info("gotoAndPlay(3)");
                    break;
                case 7:
                    stop();
                    nextScene();
                    info("stop(), nextScene() with a single scene");
                    break;
                case 8:
                    stop();
                    prevScene();
                    info("stop(), prevScene() with a single scene");
                    break;
                case 9:
                    play();
                    gotoAndStop(currentFrame);
                    info("play(), gotoAndStop(currentFrame)");
                    break;
                case 10:
                    gotoAndPlay(currentFrame);
                    info("gotoAndPlay(currentFrame)");
                    break;
                case 12:
                    removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                    break;
            }
        }
    }
}
