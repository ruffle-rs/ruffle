package flash.filesystem {
    import flash.net.FileReference;

    [API("661")]
    public class File extends FileReference {
        private static var _applicationDirectory:File = null;
        private static var _applicationStorageDirectory:File = null;
        private static var _documentsDirectory:File = null;
        private static var _userDirectory:File = null;

        private var _nativePath:String = "";

        public function File(path:String = null) {
            super();
            if (path != null) {
                if (path.indexOf("file://") == 0) {
                    this._nativePath = path.substring(7);
                } else {
                    this._nativePath = path;
                }
            }
        }

        public static function get applicationDirectory():File {
            if (_applicationDirectory === null) {
                _applicationDirectory = new File("/");
            }
            return _applicationDirectory;
        }

        public static function get applicationStorageDirectory():File {
            if (_applicationStorageDirectory === null) {
                _applicationStorageDirectory = new File("/storage");
            }
            return _applicationStorageDirectory;
        }

        public static function get documentsDirectory():File {
            if (_documentsDirectory === null) {
                _documentsDirectory = new File("/documents");
            }
            return _documentsDirectory;
        }

        public static function get userDirectory():File {
            if (_userDirectory === null) {
                _userDirectory = new File("/user");
            }
            return _userDirectory;
        }

        public static function get separator():String {
            return "/";
        }

        public function resolvePath(path:String):File {
            if (path == null) {
                Error.throwError(TypeError, 2007, "path");
            }
            if (path.length > 0 && path.charAt(0) == "/") {
                return new File(path);
            }
            var base:String = this._nativePath;
            if (base.length > 0 && base.charAt(base.length - 1) != "/") {
                base += "/";
            }
            return new File(base + path);
        }

        public function get exists():Boolean {
            return false;
        }

        public function get nativePath():String {
            return this._nativePath;
        }

        public function set nativePath(value:String):void {
            this._nativePath = value != null ? value : "";
        }

        public function get url():String {
            return this._nativePath.length > 0 ? "file://" + this._nativePath : "";
        }

        public function set url(value:String):void {
            if (value != null && value.indexOf("file://") == 0) {
                this._nativePath = value.substring(7);
            } else {
                this._nativePath = value != null ? value : "";
            }
        }

        public function canonicalize():void {
        }

        public function createDirectory():void {
        }

        public function deleteDirectory(deleteDirectoryContents:Boolean = false):void {
        }

        public function deleteFile():void {
        }

        public function getDirectoryListing():Array {
            return [];
        }

        public function moveTo(newLocation:FileReference, overwrite:Boolean = false):void {
        }

        public function copyTo(newLocation:FileReference, overwrite:Boolean = false):void {
        }
    }
}
