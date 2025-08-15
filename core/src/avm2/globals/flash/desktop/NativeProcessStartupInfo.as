package flash.desktop {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    import flash.filesystem.File;

    [API("668")]
    public class NativeProcessStartupInfo {
        public function NativeProcessStartupInfo() {
            super();
        }

        public function get arguments():Vector.<String> {
            stub_getter("flash.desktop.NativeProcessStartupInfo", "arguments");

            return null;
        }

        public function set arguments(value:Vector.<String>):void {
            stub_setter("flash.desktop.NativeProcessStartupInfo", "arguments");
        }

        public function get executable():File {
            stub_getter("flash.desktop.NativeProcessStartupInfo", "executable");

            return null;
        }

        public function set executable(value:File):void {
            stub_setter("flash.desktop.NativeProcessStartupInfo", "executable");
        }

        public function get workingDirectory():File {
            stub_getter("flash.desktop.NativeProcessStartupInfo", "workingDirectory");

            return null;
        }

        public function set workingDirectory(value:File):void {
            stub_setter("flash.desktop.NativeProcessStartupInfo", "workingDirectory");
        }
    }
}
