package {
    import flash.display.MovieClip;
    import flash.events.Event;
    public class Test extends MovieClip {
        public function Test() {
            trace("ctor: bytesLoaded=" + loaderInfo.bytesLoaded + " bytesTotal=" + loaderInfo.bytesTotal);
            loaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                trace("complete: bytesLoaded=" + loaderInfo.bytesLoaded + " bytesTotal=" + loaderInfo.bytesTotal);
            });
            loaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                trace("init: bytesLoaded=" + loaderInfo.bytesLoaded + " bytesTotal=" + loaderInfo.bytesTotal);
            });
        }
    }
}
