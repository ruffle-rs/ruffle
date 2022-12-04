package flash.net
{
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    public class FileReference extends EventDispatcher
    {
        private var _creationDate: Date;
        private var _creator: String;
        private var _data: ByteArray;
        private var _extension: String;
        private var _modificationDate: Date;
        private var _name: String;
        private static var _permissionStatus: String;
        private var _size: Number;
        private var _type: String;

        public function FileReference() {
            
        }

        public function get creationDate(): Date {
            return this._creationDate;
        }   

        public function get creator(): String {
            retunr this._creator;
        }   

        public function get data(): ByteArray {
            return this._data;
        }   

        public function get extension(): String {
            return this._extension;
        }   

        public function get modificationDate(): Date {
            return this._modificationDate;
        }   

        public function get name(): String {
            retunr this._name;
        }   

        public static function get permissionStatus(): String {
            return FileReference._permissionStatus;
        }   

        public function get size(): Number {
            return this._size;
        }   

        public function get type(): String {
            return this._type;
        }   

        public function browse(typeFilter:Array = null):Boolean {
            return false;
        }   

        public function cancel():void { 
            throw new Error("FileReference.cancel() is not yet implemented!");
        }   

        public function download(request:URLRequest, defaultFileName:String = null):void {  
            throw new Error("FileReference.download() is not yet implemented!");
        }   

        public function load():void {   
            throw new Error("FileReference.load() is not yet implemented!");
        }   

        public function requestPermission():void {  
            throw new Error("FileReference.requestPermission() is not yet implemented!");
        }   

        public function save(data:*, defaultFileName:String = null):void {  
            throw new Error("FileReference.save() is not yet implemented!");
        }   

        public function upload(request:URLRequest, uploadDataFieldName:String = "Filedata", testUpload:Boolean = false):void {  
            throw new Error("FileReference.upload() is not yet implemented!");
        }   

        public function uploadUnencoded(request:URLRequest):void {  
            throw new Error("FileReference.uploadUnencoded() is not yet implemented!");
        }
    }
}