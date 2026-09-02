package {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.net.URLRequest;
    import flash.system.ApplicationDomain;
    import flash.system.LoaderContext;

    // The pattern a game uses for equipment: load an asset SWF into a child
    // domain, take a linked class out of that domain, throw the loaded
    // content away, and instantiate the class later. The class is what stays
    // reachable - not the Loader and not the loaded content - so the player
    // must keep the movie's characters for as long as the class is held, and
    // must let all of it go once the class is released.
    public class Test extends MovieClip {
        private static const INSTANTIATE_AT:int = 90;
        private static const RELEASE_AT:int = 150;
        private static const DONE_AT:int = 230;

        private var loader:Loader;
        private var domain:ApplicationDomain;
        private var heldClass:Class = null;
        private var frame:int = 0;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
            domain = new ApplicationDomain(ApplicationDomain.currentDomain);
            loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onLoaded);
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            addChild(loader);
            loader.load(new URLRequest("./child/child.swf"), new LoaderContext(false, domain));
        }

        private function onLoaded(e:Event):void {
            try {
                heldClass = domain.getDefinition("Child") as Class;
            } catch (err:Error) {
                trace("getDefinition failed: " + err);
            }
            trace("loaded, class held: " + (heldClass != null));

            // Discard the content but keep the class.
            loader.contentLoaderInfo.removeEventListener(Event.COMPLETE, onLoaded);
            loader.contentLoaderInfo.removeEventListener(IOErrorEvent.IO_ERROR, onError);
            loader.unloadAndStop(true);
            if (loader.parent != null) {
                loader.parent.removeChild(loader);
            }
            loader = null;
            trace("unloaded");
        }

        private function onError(e:IOErrorEvent):void {
            trace("error " + e.text);
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == INSTANTIATE_AT) {
                instantiate();
            } else if (frame == RELEASE_AT) {
                heldClass = null;
                domain = null;
                trace("released");
            } else if (frame == DONE_AT) {
                trace("done");
            }
        }

        private function instantiate():void {
            if (heldClass == null) {
                trace("no class to instantiate");
                return;
            }
            try {
                var inst:Object = new heldClass();
                var children:int = -1;
                if (inst is DisplayObjectContainer) {
                    children = DisplayObjectContainer(inst).numChildren;
                }
                var width:int = (inst is DisplayObject) ? int(DisplayObject(inst).width) : -1;
                trace("instantiated children=" + children + " width=" + width);
            } catch (err:Error) {
                trace("instantiation threw " + err);
            }
        }
    }
}
