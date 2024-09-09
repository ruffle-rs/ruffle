package flash.display {
    import flash.events.EventDispatcher;
    import flash.system.ApplicationDomain;
    import flash.utils.ByteArray;
    import flash.events.UncaughtErrorEvents;

    [Ruffle(InstanceAllocator)]
    [Ruffle(SuperInitializer)]
    public class LoaderInfo extends EventDispatcher {
        public function LoaderInfo() {
            throw new Error("LoaderInfo cannot be constructed");
        }

        public native function get actionScriptVersion():uint;
        public native function get applicationDomain():ApplicationDomain;
        public native function get bytesLoaded():uint;
        public native function get bytesTotal():uint;
        public native function get content():DisplayObject;
        public native function get contentType():String;
        public native function get frameRate():Number;
        public native function get height():int;
        public native function get isURLInaccessible():Boolean;
        public native function get parentAllowsChild():Boolean;
        public native function get swfVersion():uint;
        public native function get url():String;
        public native function get width():int;
        public native function get bytes():ByteArray;
        public native function get loader():Loader;
        public native function get loaderURL():String;
        public native function get parameters():Object;
        public native function get sharedEvents():EventDispatcher;
        public native function get uncaughtErrorEvents():UncaughtErrorEvents;
        public native function get sameDomain():Boolean;
        public native function get childAllowsParent():Boolean;
    }
}
