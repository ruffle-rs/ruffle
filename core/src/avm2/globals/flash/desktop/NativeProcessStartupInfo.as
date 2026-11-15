package flash.desktop {
    import flash.filesystem.File;

    [API("668")]
    public class NativeProcessStartupInfo {
        private var _arguments:Vector.<String> = new Vector.<String>();
        private var _executable:File = null;
        private var _workingDirectory:File = null;

        public function NativeProcessStartupInfo() {
            super();
        }

        public function get arguments():Vector.<String> {
            return this._arguments;
        }

        public function set arguments(value:Vector.<String>):void {
            this._arguments = value;
        }

        public function get executable():File {
            return this._executable;
        }

        public function set executable(value:File):void {
            this._executable = value;
        }

        public function get workingDirectory():File {
            return this._workingDirectory;
        }

        public function set workingDirectory(value:File):void {
            this._workingDirectory = value;
        }
    }
}
