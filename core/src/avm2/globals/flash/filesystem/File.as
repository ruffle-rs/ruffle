package flash.filesystem {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    import flash.net.FileReference;

    [API("661")]
    public class File extends FileReference {
        private static var _applicationDirectory:File = null;

        public function File(path:String = null) {
            stub_constructor("flash.filesystem.File");
        }

        public static function get applicationDirectory():File {
            if (_applicationDirectory === null) {
                _applicationDirectory = new File();
            }

            return _applicationDirectory;
        }

        public function resolvePath(path:String):File {
            stub_method("flash.filesystem.File", "resolvePath");

            return new File();
        }

        public function get exists():Boolean {
            stub_getter("flash.filesystem.File", "exists");

            return false;
        }
    }
}
