package flash.ui {
    [Ruffle(Abstract)]
    public final class Multitouch {
        private static var _inputMode:String = MultitouchInputMode.GESTURE;
        private static var _mapTouchToMouse:Boolean = true;

        public static function get inputMode():String {
            return _inputMode;
        }

        public static function set inputMode(value:String):void {
            if (value !== MultitouchInputMode.NONE &&
                value !== MultitouchInputMode.GESTURE &&
                value !== MultitouchInputMode.TOUCH_POINT) {
                Error.throwError(ArgumentError, 2008, "inputMode");
            }
            _inputMode = value;
        }

        [API("675")]
        public static function get mapTouchToMouse():Boolean {
            return _mapTouchToMouse;
        }

        [API("675")]
        public static function set mapTouchToMouse(value:Boolean):void {
            _mapTouchToMouse = value;
        }

        public static function get maxTouchPoints():int {
            return 0;
        }

        public static function get supportedGestures():Vector.<String> {
            return null;
        }

        public static function get supportsGestureEvents():Boolean {
            return false;
        }

        public static function get supportsTouchEvents():Boolean {
            return false;
        }
    }
}
