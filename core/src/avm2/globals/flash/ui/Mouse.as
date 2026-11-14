package flash.ui {
    [Ruffle(Abstract)]
    public final class Mouse {
        public static native function hide():void;
        public static native function show():void;

        public static native function get cursor():String;
        public static native function set cursor(value:String):void;

        public static function get supportsCursor():Boolean {
            return true;
        }
        public static function get supportsNativeCursor():Boolean {
            return true;
        }

        public static native function registerCursor(name:String, cursor:MouseCursorData):void;
        public static native function unregisterCursor(name:String):void;
    }
}
