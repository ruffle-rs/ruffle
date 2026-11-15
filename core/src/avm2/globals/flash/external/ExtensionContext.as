package flash.external {
    import flash.events.EventDispatcher;

    [API("669")]
    public final class ExtensionContext extends EventDispatcher {
        public var actionScriptData:Object;

        public function ExtensionContext() {
            super();
        }

        public static function createExtensionContext(extensionID:String, contextType:String):ExtensionContext {
            return new ExtensionContext();
        }

        public function call(functionName:String, ...rest):Object {
            return null;
        }

        public function dispose():void {
        }
    }
}

