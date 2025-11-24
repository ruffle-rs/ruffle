package flash.system {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    import flash.events.EventDispatcher;

    [Ruffle(Abstract)]
    public final class IME extends EventDispatcher {
        private static var _conversionMode:String = "ALPHANUMERIC_HALF";
        private static var _enabled:Boolean;
        private static var _isSupported:Boolean;

        public static function compositionAbandoned():void {
            stub_method("flash.system.IME", "compositionAbandoned");
        }

        public static function compositionSelectionChanged(start:int, end:int):void {
            stub_method("flash.system.IME", "compositionSelectionChanged");
        }

        public static function doConversion():void {
            stub_method("flash.system.IME", "doConversion");
        }

        public static function setCompositionString(composition:String):void {
            stub_method("flash.system.IME", "setCompositionString");
        }

        public function get isSupported():Boolean {
            return _isSupported;
        }

        public static function get enabled():Boolean {
            stub_getter("flash.system.IME", "enabled");
            return _enabled;
        }

        public static function set enabled(value:Boolean):void {
            stub_setter("flash.system.IME", "enabled");
            _enabled = value;
        }

        public static function get conversionMode():String {
            stub_getter("flash.system.IME", "conversionMode");
            return _conversionMode;
        }

        public static function set conversionMode(value:String):void {
            stub_setter("flash.system.IME", "conversionMode");
            _conversionMode = value;
        }
    }
}
