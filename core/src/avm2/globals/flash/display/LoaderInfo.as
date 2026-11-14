package flash.display {
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.events.UncaughtErrorEvents;
    import flash.errors.IllegalOperationError;
    import flash.system.ApplicationDomain;
    import flash.utils.ByteArray;

    [Ruffle(InstanceAllocator)]
    public class LoaderInfo extends EventDispatcher {
        private var _parentSandboxBridge:Object = null;
        private var _childSandboxBridge:Object = null;

        public native function get actionScriptVersion():uint;

        public native function get applicationDomain():ApplicationDomain;

        public native function get bytes():ByteArray;

        public native function get bytesLoaded():uint;

        public native function get bytesTotal():uint;

        public native function get childAllowsParent():Boolean;

        public function get childSandboxBridge():Object {
            return this._childSandboxBridge;
        }
        public function set childSandboxBridge(value:Object):void {
            this._childSandboxBridge = value;
        }

        public native function get content():DisplayObject;

        public native function get contentType():String;

        public native function get frameRate():Number;

        public native function get height():int;

        [API("674")]
        public native function get isURLInaccessible():Boolean;

        public native function get loader():Loader;

        public native function get loaderURL():String;

        public native function get parameters():Object;

        public native function get parentAllowsChild():Boolean;

        public function get parentSandboxBridge():Object {
            return this._parentSandboxBridge;
        }
        public function set parentSandboxBridge(value:Object):void {
            this._parentSandboxBridge = value;
        }

        public native function get sameDomain():Boolean;

        public native function get sharedEvents():EventDispatcher;

        public native function get swfVersion():uint;

        public native function get uncaughtErrorEvents():UncaughtErrorEvents;

        public native function get url():String;

        public native function get width():int;

        override public function dispatchEvent(event:Event):Boolean {
            throw new IllegalOperationError("Error #2118: The LoaderInfo class does not implement this method.", 2118);
        }

        public static function getLoaderInfoByDefinition(object:Object):LoaderInfo {
            if (object == null) {
                Error.throwError(TypeError, 2007, "object");
            }
            return null;
        }
    }
}
