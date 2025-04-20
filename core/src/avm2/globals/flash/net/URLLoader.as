package flash.net {
    import flash.events.EventDispatcher;
    import flash.net.URLRequest;
    import __ruffle__.stub_method;

    public class URLLoader extends EventDispatcher {
        [Ruffle(NativeAccessible)]
        public var data: *;

        [Ruffle(NativeAccessible)]
        public var dataFormat: String = "text";

        [Ruffle(NativeAccessible)]
        public var bytesLoaded:uint;

        [Ruffle(NativeAccessible)]
        public var bytesTotal:uint;

        public function URLLoader(request:URLRequest = null) {
            if (request != null) {
                this.load(request);
            }
        }

        public native function load(request:URLRequest):void;

        public function close():void {
            stub_method("flash.net.URLLoader", "close");
        }
    }
}
