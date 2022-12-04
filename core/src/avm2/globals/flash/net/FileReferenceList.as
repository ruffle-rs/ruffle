package flash.net
{
    import flash.events.EventDispatcher;
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
            throw new Error("FileReferenceList.browse() is not yet implemented!");
        }
    }
}