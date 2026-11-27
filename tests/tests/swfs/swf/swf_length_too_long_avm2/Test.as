// The SWF header declares more bytes than the file has (see patch.r2). In AVM2,
// Flash Player reports the actual size and dispatches init, progress and
// complete as usual, but after the last frame it waits for the missing frames
// instead of looping back to the first one.
package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;

    public class Test extends MovieClip {
        private var ticks:int = 0;

        public function Test() {
            info("constructor");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                trace("progress " + e.bytesLoaded + "/" + e.bytesTotal);
            });
            loaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                info("init");
            });
            loaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                info("complete");
            });
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                ticks++;

                if (ticks <= 8) {
                    info("enterFrame " + ticks);
                }
            });
        }

        private function info(label:String):void {
            trace(label + ": frame " + currentFrame + "/" + totalFrames + ", framesLoaded " + framesLoaded + ", bytes " + loaderInfo.bytesLoaded + "/" + loaderInfo.bytesTotal);
        }
    }
}
