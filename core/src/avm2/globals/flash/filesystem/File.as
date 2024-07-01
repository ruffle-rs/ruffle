package flash.filesystem
{
    import flash.net.FileReference;

    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    public class File extends FileReference
    {
        private native function init(path: String);

        public function File(path: String = null) {
            // AIR supports both native and other types of path.
            this.init(path);
        }

        public static function get applicationDirectory(): File {
            return new File("app:");
        }

        public static function get applicationStorageDirectory(): File {
            return new File("app-storage:");
        }

        public native function get exists(): Boolean;
        public native function get isDirectory(): Boolean;
        public function get isHidden(): Boolean {
            stub_getter("flash.filesystem.File", "isHidden");
            return false;
        }
        public native function get nativePath(): String;
        public native function get url(): String;

        public native function createDirectory():void;
        public native function getDirectoryListing(): Array;
        public function getDirectoryListingAsync(): void {
            stub_getter("flash.filesystem.File", "getDirectoryListingAsync");
        }
        public native function resolvePath(path): File;
    }
}