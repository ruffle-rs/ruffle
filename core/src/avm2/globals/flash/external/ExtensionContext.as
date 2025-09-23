package flash.external {
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    [API("669")]
    public final class ExtensionContext extends EventDispatcher {
        public function ExtensionContext() {
            super();
        }

        public static function createExtensionContext(extensionID:String, contextType:String):ExtensionContext {
            stub_method("flash.external.ExtensionContext", "createExtensionContext");

            return new ExtensionContext();
        }

        public function call(functionName:String, ...rest):Object {
            stub_method("flash.external.ExtensionContext", "call");

            return null;
        }
    }
}
