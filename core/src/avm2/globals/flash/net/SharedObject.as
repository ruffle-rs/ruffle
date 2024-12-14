package flash.net {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;

    namespace ruffle = "__ruffle__";

    [Ruffle(InstanceAllocator)]
    public class SharedObject extends EventDispatcher {
        public function SharedObject() {
           // Unreachable; the allocator always throws
        }

        // NOTE: We currently always use AMF3 serialization.
        // If you implement the `defaultObjectEncoding` or `objectEncoding`,
        // you will need to adjust the serialization and deserialization code
        // to work with AMF0.

        public static native function getLocal(name:String, localPath:String = null, secure:Boolean = false): SharedObject;

        public native function get size() : uint;
        public native function get objectEncoding() : uint;
        public native function set objectEncoding(value:uint) : void;

        public native function flush(minDiskSpace:int = 0) : String;
        public native function close() : void;
        public native function clear() : void;

        public function setProperty(propertyName:String, value:Object = null):void {
            this.data[propertyName] = value;
            // This should also mark remote SharedObjects as dirty,
            // but we don't support them yet
        }

        public native function get data():Object;
    }
}
