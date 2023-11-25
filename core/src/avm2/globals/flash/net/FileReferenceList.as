package flash.net
{
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;

    public class FileReferenceList extends EventDispatcher
    {
        private var _fileList: Array;
        private var _file: FileReference;

        public function FileReferenceList() {
            var self = this;

            this._file = new FileReference();
            this._file.addEventListener(Event.SELECT, function(e:*): void {
                self._fileList[0] = self._file;
                self.dispatchEvent(new Event(Event.SELECT));
            });
            this._file.addEventListener(Event.CANCEL, function(e:*): void {
                self.dispatchEvent(new Event(Event.CANCEL));
            });
        }

        public function get fileList(): Array {
            return this._fileList;
        }

        public function browse(typeFilter: Array = null): Boolean {
            stub_method("flash.net.FileReferenceList", "browse");

            this._fileList = [];
            return this._file.browse(typeFilter);
        }
    }
}
