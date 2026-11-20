package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.net.URLRequest;
    import flash.system.ApplicationDomain;
    import flash.system.LoaderContext;

    public class Test extends Sprite {
        private var childLoader:Loader = new Loader();
        private var lateLoader:Loader = new Loader();
        private var lookupLoader:Loader = new Loader();
        private var child:Object;
        private var rootDomain:ApplicationDomain = ApplicationDomain.currentDomain;

        public function Test() {
            trace("BEGIN DOMAIN_MULTINAME_LOOKUP");
            trace("parent Choice=" + rootDomain.getDefinition("parentns.Choice").NAME);
            trace("parent has Collision before child=" + rootDomain.hasDefinition("shared.Collision"));
            childLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onChild);
            childLoader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            childLoader.load(new URLRequest("child.swf"),
                new LoaderContext(false, new ApplicationDomain(rootDomain)));
        }

        private function onChild(event:Event):void {
            lookupLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onLookup);
            lookupLoader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            lookupLoader.load(new URLRequest("lookup.swf"),
                new LoaderContext(false, childLoader.contentLoaderInfo.applicationDomain));
        }

        private function onLookup(event:Event):void {
            child = lookupLoader.content;
            trace("=== before late parent definition ===");
            child.warmUntaken();
            child.run("first");
            child.run("repeat");
            trace("parent has Collision after child=" + rootDomain.hasDefinition("shared.Collision"));
            lateLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onLate);
            lateLoader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            lateLoader.load(new URLRequest("late.swf"), new LoaderContext(false, rootDomain));
        }

        private function onLate(event:Event):void {
            trace("=== after late parent definition ===");
            trace("parent Collision=" + rootDomain.getDefinition("shared.Collision").NAME);
            child.readAfterLate();
            child.run("after late");
            child.run("after late repeat");
            trace("DOMAIN_MULTINAME_LOOKUP_COMPLETE");
        }

        private function onError(event:IOErrorEvent):void {
            trace("LOAD_FAILED " + event.type);
        }
    }
}
