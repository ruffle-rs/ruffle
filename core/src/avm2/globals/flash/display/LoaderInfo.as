package flash.display {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import __ruffle__.stub_setter;

    import flash.errors.IllegalOperationError;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.events.UncaughtErrorEvents;
    import flash.system.ApplicationDomain;
    import flash.utils.ByteArray;

    [Ruffle(Abstract)]
    public class LoaderInfo extends EventDispatcher {
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
        public native function get sameDomain():Boolean;
        public native function get childAllowsParent():Boolean;

        [API("667")]
        public native function get uncaughtErrorEvents():UncaughtErrorEvents;

        override public function dispatchEvent(event:Event):Boolean {
            throw new IllegalOperationError("Error #2118: The LoaderInfo class does not implement this method.", 2118);
        }

        public static function getLoaderInfoByDefinition(object:Object):LoaderInfo {
            // Docs say that this returns `null` when debugging isn't enabled,
            // and (TODO) in FP it throws a SecurityError unless called in a
            // local SWF that has been marked as "trusted", so do we really need
            // to implement it?
            stub_method("flash.display.LoaderInfo", "getLoaderInfoByDefinition");
            return null;
        }

        // Playerglobals says all the *Bridge functions are AIR-only, but that
        // doesn't seem to be true

        public function get parentSandboxBridge():Object {
            stub_getter("flash.display.LoaderInfo", "parentSandboxBridge");
            return null;
        }
        public function set parentSandboxBridge(obj:Object):void {
            stub_setter("flash.display.LoaderInfo", "parentSandboxBridge");
        }

        public function get childSandboxBridge():Object {
            stub_getter("flash.display.LoaderInfo", "childSandboxBridge");
            return null;
        }
        public function set childSandboxBridge(obj:Object):void {
            stub_setter("flash.display.LoaderInfo", "childSandboxBridge");
        }
    }
}
