package flash.net
{
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class FileReference extends EventDispatcher
    {
        private var _creationDate: Date;
        private var _creator: String;
        private var _extension: String;
        private var _modificationDate: Date;
        private static var _permissionStatus: String;
        private var _type: String;

        public function FileReference() {
            
        }

        public function get creationDate(): Date {
            return this._creationDate;
        }   

        public function get creator(): String {
            return this._creator;
        }   

        public native function get data(): ByteArray;

        public function get extension(): String {
            return this._extension;
        }   

        public function get modificationDate(): Date {
            return this._modificationDate;
        }   

        public native function get name(): String;

        public static function get permissionStatus(): String {
            return FileReference._permissionStatus;
        }   

        public native function get size(): Number;

        public function get type(): String {
            return this._type;
        }   

        public native function browse(typeFilter:Array = null): Boolean;

        public function cancel():void {
            stub_method("flash.net.FileReference", "cancel");
        }   

        public function download(request:URLRequest, defaultFileName:String = null):void {
            stub_method("flash.net.FileReference", "download");
        }   

        public native function load():void;

        public function requestPermission():void {
            stub_method("flash.net.FileReference", "requestPermission");
        }   

        public native function save(data:*, defaultFileName:String = null):void;

        public function upload(request:URLRequest, uploadDataFieldName:String = "Filedata", testUpload:Boolean = false):void {
            stub_method("flash.net.FileReference", "upload");
        }   

        public function uploadUnencoded(request:URLRequest):void {
            stub_method("flash.net.FileReference", "uploadUnencoded");
        }
    }
}