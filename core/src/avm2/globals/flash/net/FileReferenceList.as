package flash.net
{
    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;

    public class FileReferenceList extends EventDispatcher
    {
        private var _fileList:Array;
        public function FileReferenceList()
        {
            _fileList = new Array();
        }

        public function get fileList():Array
        {
            return this._fileList;
        }

        public function browse(typeFilter:Array = null):Boolean
        {
            stub_method("flash.net.FileReferenceList", "browse");
            return false;
        }
    }
}