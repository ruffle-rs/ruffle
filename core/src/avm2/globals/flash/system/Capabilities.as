package flash.system {
    import __ruffle__.stub_getter;
    public final class Capabilities {
        public native static function get os(): String;
        public native static function get playerType(): String;
        public native static function get version(): String;
        public native static function get screenResolutionX():Number;
        public native static function get screenResolutionY():Number;
        public native static function get pixelAspectRatio():Number;
        public native static function get screenDPI():Number;
        public static function get manufacturer(): String {
            stub_getter("flash.system.Capabilities", "manufacturer");
            return "Adobe Windows"
        }
        public static function get language(): String {
            stub_getter("flash.system.Capabilities", "language");
            return "en"
        }
        public static function get isDebugger(): Boolean {
            return false
        }
    }
}
