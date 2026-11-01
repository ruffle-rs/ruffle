// compiled with mxmlc

package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.ProgressEvent;

    // A preloader on the first frame gets the root movie's last `progress`
    // event, between `init` and `complete`.
    public class Test extends Sprite {
        public function Test() {
            trace("first frame");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, onProgress);
            loaderInfo.addEventListener(Event.INIT, onEvent);
            loaderInfo.addEventListener(Event.COMPLETE, onEvent);
        }

        private function onProgress(event:ProgressEvent):void {
            trace("progress: " + event.bytesLoaded + "/" + event.bytesTotal);
        }

        private function onEvent(event:Event):void {
            trace(event.type);
        }
    }
}
