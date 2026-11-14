package flash.accessibility {
    import flash.display.DisplayObject;

    [Ruffle(Abstract)]
    public final class Accessibility {
        private static var _active:Boolean = false;

        public static function sendEvent(source:DisplayObject, childID:uint, eventType:uint, nonHTML:Boolean = false):void {
            if (source == null) {
                Error.throwError(TypeError, 2007, "source");
            }
            if (!_active) {
                return;
            }
        }

        public static function updateProperties():void {
            if (!_active) {
                return;
            }
        }

        public static function get active():Boolean {
            return _active;
        }
    }
}
