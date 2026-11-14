package flash.text {
    public final class TextRenderer {
        private static var _displayMode:String = "default";
        private static var _maxLevel:int = 4;

        public static function setAdvancedAntiAliasingTable(
            fontName:String,
            fontStyle:String,
            colorType:String,
            advancedAntiAliasingTable:Array
        ):void {
        }

        public static function get displayMode():String {
            return _displayMode;
        }

        public static function set displayMode(value:String):void {
            _displayMode = value;
        }

        public static function get maxLevel():int {
            return _maxLevel;
        }

        public static function set maxLevel(value:int):void {
            _maxLevel = value;
        }
    }
}
