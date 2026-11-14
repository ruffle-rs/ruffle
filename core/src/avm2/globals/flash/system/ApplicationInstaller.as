package flash.system {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;

    [API("680")]
    public final class ApplicationInstaller extends EventDispatcher {
        public function ApplicationInstaller() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public static function stringsDigest(strings:XML):String {
            return "";
        }

        public function install(shortcutsOnly:Boolean = false):void {
        }
    }
}
