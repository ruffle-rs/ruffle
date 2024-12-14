package flash.desktop {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import flash.system.System;

    public class Clipboard {
        private static var _generalClipboard = new Clipboard();

        public static function get generalClipboard(): Clipboard {
            return Clipboard._generalClipboard;
        }

        function Clipboard() {
            // TODO: This should only be callable in AIR
        }

        public function get formats(): Array {
            stub_getter("flash.desktop.Clipboard", "formats");
            return new Array();
        }

        public function clear(): void {
            stub_method("flash.desktop.Clipboard", "clear");
        }

        public function clearData(format: String): void {
            stub_method("flash.desktop.Clipboard", "clearData");
        }

        public function getData(format: String, transferMode: String = ClipboardTransferMode.ORIGINAL_PREFERRED): Object {
            stub_method("flash.desktop.Clipboard", "getData");
            return null;
        }

        public function hasFormat(format: String): Boolean {
            stub_method("flash.desktop.Clipboard", "hasFormat");
            return false;
        }

        public function setData(format: String, data: Object, serializable: Boolean = true): Boolean {
            stub_method("flash.desktop.Clipboard", "setData");
            if (format == ClipboardFormats.TEXT_FORMAT) {
                System.setClipboard(data);
                return true;
            }
            return false;
        }

        public function setDataHandler(format: String, handler: Function, serializable: Boolean = true): Boolean {
            stub_method("flash.desktop.Clipboard", "setDataHandler");
            return false;
        }
    }
}
