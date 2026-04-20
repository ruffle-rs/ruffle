package flash.filesystem {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    import flash.net.FileReference;

    [API("661")]
    public class File extends FileReference {
        private static var _applicationDirectory:File = null;
        private static var _applicationStorageDirectory:File = null;
        private static var _documentsDirectory:File = null;
        private static var _userDirectory:File = null;

        public function File(path:String = null) {
            stub_constructor("flash.filesystem.File");
        }

        public static function get applicationDirectory():File {
            stub_getter("flash.filesystem.File", "applicationDirectory");

            if (_applicationDirectory === null) {
                _applicationDirectory = new File();
            }

            return _applicationDirectory;
        }

        public static function get applicationStorageDirectory():File {
            stub_getter("flash.filesystem.File", "applicationStorageDirectory");

            if (_applicationStorageDirectory === null) {
                _applicationStorageDirectory = new File();
            }

            return _applicationStorageDirectory;
        }

        public static function get documentsDirectory():File {
            stub_getter("flash.filesystem.File", "documentsDirectory");

            if (_documentsDirectory === null) {
                _documentsDirectory = new File();
            }

            return _documentsDirectory;
        }

        public static function get userDirectory():File {
            stub_getter("flash.filesystem.File", "userDirectory");

            if (_userDirectory === null) {
                _userDirectory = new File();
            }

            return _userDirectory;
        }

        public static function get separator():String {
            stub_getter("flash.filesystem.File", "separator");

            return "/";
        }

        public function resolvePath(path:String):File {
            stub_method("flash.filesystem.File", "resolvePath");

            return new File();
        }

        public function get exists():Boolean {
            stub_getter("flash.filesystem.File", "exists");

            return false;
        }

        public function get nativePath():String {
            stub_getter("flash.filesystem.File", "nativePath");

            return "";
        }

        public function get url():String {
            stub_getter("flash.filesystem.File", "url");

            return "";
        }

        public function canonicalize():void {
            stub_method("flash.filesystem.File", "canonicalize");
        }

        public function createDirectory():void {
            stub_method("flash.filesystem.File", "createDirectory");
        }

        public function deleteDirectory(deleteDirectoryContents:Boolean = false):void {
            stub_method("flash.filesystem.File", "deleteDirectory");
        }

        public function getDirectoryListing():Array {
            stub_method("flash.filesystem.File", "getDirectoryListing");

            return [];
        }

        public function moveTo(newLocation:FileReference, overwrite:Boolean = false):void {
            stub_method("flash.filesystem.File", "moveTo");
        }
    }
}
