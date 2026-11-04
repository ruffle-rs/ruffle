package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var loader:Loader;
        private var frames:int = 0;

        public function Test() {
            loader = new Loader();
            addChild(loader);
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            loader.contentLoaderInfo.addEventListener(Event.UNLOAD, onUnload);
            loader.load(new URLRequest("test_image.jpg"));
        }

        private function onComplete(event:Event):void {
            trace("COMPLETE");
            trace("PARENT_BEFORE_NULL=" + (loader.parent == null));
            trace("STAGE_BEFORE_NULL=" + (loader.stage == null));
            trace("CONTENT_BEFORE_NULL=" + (loader.content == null));
            trace("WIDTH_BEFORE=" + loader.width);
            trace("HEIGHT_BEFORE=" + loader.height);

            addEventListener(Event.EXIT_FRAME, onDelayedUnload);
        }

        private function onDelayedUnload(event:Event):void {
            frames++;
            if (frames < 3) {
                return;
            }
            removeEventListener(Event.EXIT_FRAME, onDelayedUnload);
            trace("DELAYED_UNLOAD");
            loader.unload();

            trace("CONTENT_AFTER_NULL=" + (loader.content == null));
            trace("STAGE_AFTER_NULL=" + (loader.stage == null));
            trace("WIDTH_AFTER=" + loader.width);
            trace("HEIGHT_AFTER=" + loader.height);
        }

        private function onUnload(event:Event):void {
            trace("UNLOAD_EVENT");
        }
    }
}
