package flash.ui {

    import __ruffle__.stub_getter;

    public final class Mouse {
        public static native function hide(): void;
        public static native function show(): void;
        public static function get supportsCursor():Boolean {
            stub_getter("flash.ui.Mouse", "supportsCursor");
            return true;
        }
        public static function get supportsNativeCursor():Boolean {
            stub_getter("flash.ui.Mouse", "supportsNativeCursor");
            return true;
        }
    }
}
