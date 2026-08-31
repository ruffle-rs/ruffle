package {
    import flash.display.Loader;
    import flash.display.LoaderInfo;
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.HTTPStatusEvent;
    import flash.events.IOErrorEvent;
    import flash.events.ProgressEvent;
    import flash.events.SecurityErrorEvent;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        private var loader:Loader = new Loader();
        private var reloaded:Boolean = false;
        private var frameNum:int = 0;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onEnterFrame);

            loader.addEventListener(Event.ADDED, onEvent);
            loader.addEventListener(Event.ADDED_TO_STAGE, onEvent);
            loader.addEventListener(Event.REMOVED, onEvent);
            loader.addEventListener(Event.REMOVED_FROM_STAGE, onEvent);

            var info:LoaderInfo = loader.contentLoaderInfo;
            info.addEventListener(Event.OPEN, onEvent);
            info.addEventListener(ProgressEvent.PROGRESS, onProgress);
            info.addEventListener(Event.INIT, onEvent);
            info.addEventListener(Event.COMPLETE, onComplete);
            info.addEventListener(Event.UNLOAD, onEvent);
            info.addEventListener(IOErrorEvent.IO_ERROR, onEvent);
            info.addEventListener(SecurityErrorEvent.SECURITY_ERROR, onEvent);
            info.addEventListener(HTTPStatusEvent.HTTP_STATUS, onHttpStatus);

            addChild(loader);
            loader.load(new URLRequest("avm1_child.swf"));
        }

        private function onEnterFrame(event:Event):void {
            frameNum++;
            trace("frame " + frameNum);
            if (frameNum >= 6) {
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }

        private function onEvent(event:Event):void {
            trace(
                event.type +
                ": target=" + event.target +
                ", currentTarget=" + event.currentTarget +
                ", eventPhase=" + event.eventPhase +
                ", bubbles=" + event.bubbles
            );
        }

        private function onProgress(event:ProgressEvent):void {
            trace(
                event.type +
                ": bytesLoaded=" + event.bytesLoaded +
                ", bytesTotal=" + event.bytesTotal
            );
        }

        private function onHttpStatus(event:HTTPStatusEvent):void {
            trace(
                event.type +
                ": status=" + event.status
            );
        }

        private function onComplete(event:Event):void {
            onEvent(event);
            if (!reloaded) {
                reloaded = true;
                loader.load(new URLRequest("avm1_child.swf"));
            }
        }
    }
}
