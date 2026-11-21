package {
    import flash.display.MovieClip;
    import flash.display.StageAlign;
    import flash.display.StageQuality;
    import flash.display.StageScaleMode;
    import flash.events.Event;
    import flash.system.fscommand;

    [SWF(width="160", height="160", frameRate="24", backgroundColor="#A0A0A0")]
    public class Test extends MovieClip {
        private var scene:MovieClip;
        private var sampleFrame:int = 1;
        private var holdFrames:int = 0;

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;
            stage.quality = StageQuality.HIGH;

            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(event:Event):void {
            if (scene == null) {
                // The build script adds the native timeline to the root frame.
                if (numChildren == 0) {
                    return;
                }
                scene = MovieClip(getChildAt(0));
                trace("Timeline mask cache bounds");
            }
            if (holdFrames > 0) {
                holdFrames--;
                return;
            }
            scene.gotoAndStop(sampleFrame);
            holdFrames = 71;
            trace("frame=" + scene.currentFrame);
            fscommand("captureImage", "frame0" + sampleFrame);
            if (sampleFrame == scene.totalFrames) {
                removeEventListener(Event.ENTER_FRAME, onFrame);
                trace("DONE");
            }
            sampleFrame++;
        }
    }
}
