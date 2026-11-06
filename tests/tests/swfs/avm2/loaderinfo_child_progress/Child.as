package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;
    import flash.system.ApplicationDomain;

    public class Child extends MovieClip {
        private var frames:int = 0;

        private function log(s:String):void {
            ApplicationDomain.currentDomain.getDefinition("Test").log(s);
        }

        public function Child() {
            log("child constructor");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                log("child progress: " + e.bytesLoaded + " / " + e.bytesTotal);
            });
            loaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                log("child init");
            });
            loaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                log("child complete");
            });
            addEventListener(Event.EXIT_FRAME, function(e:Event):void {
                if (++frames <= 2) {
                    log("child exitFrame");
                }
            });
        }
    }
}
