package flash.accessibility {
    import __ruffle__.stub_method;
    import flash.display.DisplayObject;

    [Ruffle(Abstract)]
    public final class Accessibility {
        private static var _active:Boolean;

        public static function sendEvent(source:DisplayObject, childID:uint, eventType:uint, nonHTML:Boolean = false):void {
            stub_method("flash.accessibility.Accessibility", "sendEvent");
        }

        public static function updateProperties():void {
            stub_method("flash.accessibility.Accessibility", "updateProperties");
        }

        public static function get active():Boolean {
            return _active;
        }
    }
}
