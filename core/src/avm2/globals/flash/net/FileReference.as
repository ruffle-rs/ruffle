package flash.net
{
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class FileReference extends EventDispatcher
    {
        public function FileReference() {
        }

        public native function get creationDate(): Date;

        public function get creator(): String {
            // This was macOS (pre OS X) only. (Deprecated)
            return null;
        }

        public native function get data(): ByteArray;

        // AIR 1.0
        [API("661")]
        public function get extension(): String {
            // The file extension, excluding the dot.
            return this.type ? this.type.slice(1) : null;
        }

        public native function get modificationDate(): Date;

        public native function get name(): String;

        public native function get size(): Number;

        // File extension, including the dot. (Deprecated)
        public native function get type(): String;

        public native function browse(typeFilter:Array = null): Boolean;

        public function cancel():void {
            stub_method("flash.net.FileReference", "cancel");
        }

        public function download(request:URLRequest, defaultFileName:String = null):void {
            stub_method("flash.net.FileReference", "download");
        }

        public native function load():void;

        [API("681")]
        public function requestPermission():void {
            stub_method("flash.net.FileReference", "requestPermission");
        }

        public native function save(data:*, defaultFileName:String = null):void;

        public function upload(request:URLRequest, uploadDataFieldName:String = "Filedata", testUpload:Boolean = false):void {
            stub_method("flash.net.FileReference", "upload");
        }

        [API("681")]
        public function uploadUnencoded(request:URLRequest):void {
            stub_method("flash.net.FileReference", "uploadUnencoded");
        }
    }
}