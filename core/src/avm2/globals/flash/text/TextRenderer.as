package flash.text {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public final class TextRenderer {
        private static var _displayMode:String = "default";
        private static var _maxLevel:int = 4;

        public static function setAdvancedAntiAliasingTable(
            fontName:String,
            fontStyle:String,
            colorType:String,
            advancedAntiAliasingTable:Array
        ):void {
            stub_method("flash.text.TextRenderer", "setAdvancedAntiAliasingTable");
        }

        public static function get displayMode():String {
            stub_getter("flash.text.TextRenderer", "displayMode");
            return _displaymode;
        }

        public static function set displayMode(value:String):void {
            stub_setter("flash.text.TextRenderer", "displayMode");
            _displayMode = value;
        }

        public static function get maxLevel():int {
            stub_getter("flash.text.TextRenderer", "maxLevel");
            return _maxLevel;
        }

        public static function set maxLevel(value:int):void {
            stub_setter("flash.text.TextRenderer", "maxLevel");
            _maxLevel = value;
        }
    }
}
