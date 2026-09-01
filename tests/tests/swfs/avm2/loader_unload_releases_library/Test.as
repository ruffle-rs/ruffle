package {
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.net.URLRequest;
    import flash.system.ApplicationDomain;
    import flash.system.LoaderContext;

    // Loads and unloads the same child SWF several times over, finishing each
    // cycle before starting the next. By the time this movie traces "done",
    // every child it loaded has been unloaded and dropped, so the player must
    // not still be holding any of them.
    public class Test extends MovieClip {
        private static const CYCLES:int = 10;

        private var cycle:int = 0;
        private var loader:Loader = null;
        private var unloadPending:Boolean = false;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
            startCycle();
        }

        private function onFrame(e:Event):void {
            // Unload on the frame after the load settles, rather than from
            // inside the event handler itself.
            if (unloadPending) {
                unloadPending = false;
                finishCycle();
            }
        }

        private function startCycle():void {
            loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            addChild(loader);
            loader.load(
                new URLRequest("./child/child.swf"),
                new LoaderContext(false, new ApplicationDomain(ApplicationDomain.currentDomain))
            );
        }

        private function onComplete(e:Event):void {
            trace("loaded child " + cycle);
            unloadPending = true;
        }

        private function onError(e:IOErrorEvent):void {
            trace("error " + e.text);
            unloadPending = true;
        }

        private function finishCycle():void {
            loader.contentLoaderInfo.removeEventListener(Event.COMPLETE, onComplete);
            loader.contentLoaderInfo.removeEventListener(IOErrorEvent.IO_ERROR, onError);
            loader.unloadAndStop(true);
            if (loader.parent != null) {
                loader.parent.removeChild(loader);
            }
            loader = null;

            cycle++;
            if (cycle < CYCLES) {
                startCycle();
            } else {
                trace("done");
            }
        }
    }
}
