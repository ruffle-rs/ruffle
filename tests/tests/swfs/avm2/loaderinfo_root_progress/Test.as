package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;

    // Preloaders often stop() in frame 1 and only play() once a 'progress' event
    // reports the whole movie loaded. Flash fires that event for the root movie
    // after frame 1 has run, even when the movie was already fully loaded.
    public class Test extends MovieClip {
        public function Test() {
            trace("constructor");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                trace("progress: " + e.bytesLoaded + " / " + e.bytesTotal);
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
