package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;

    // Preloaders often leave the loading screen with a goto from their 'progress'
    // handler. That goto runs a nested frame, which must not fire the root
    // movie's final 'progress' event again.
    public class Test extends MovieClip {
        private var progressEvents:int = 0;

        public function Test() {
            trace("constructor");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                trace("progress: " + e.bytesLoaded + " / " + e.bytesTotal);
                // Stop here rather than recursing forever if it is fired again.
                if (++progressEvents > 3) {
                    return;
                }
                trace("gotoAndStop(1)");
                gotoAndStop(1);
                trace("gotoAndStop(1) done");
            });
            loaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                trace("init");
            });
            loaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                trace("complete");
            });
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                trace("enterFrame");
            });
            addEventListener(Event.EXIT_FRAME, function(e:Event):void {
                trace("exitFrame");
            });
        }
    }
}
