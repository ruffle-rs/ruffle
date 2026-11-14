package flash.text {
    [Ruffle(Abstract)]
    public class TextSnapshot {
        public function get charCount():int {
            return 0;
        }

        public function findText(beginIndex:int, textToFind:String, caseSensitive:Boolean):int {
            return -1;
        }

        public function getSelected(beginIndex:int, endIndex:int):Boolean {
            return false;
        }

        public function getSelectedText(includeLineEndings:Boolean = false):String {
            return "";
        }

        public function getText(beginIndex:int, endIndex:int, includeLineEndings:Boolean = false):String {
            return "";
        }

        public function getTextRunInfo(beginIndex:int, endIndex:int):Array {
            return [];
        }

        public function hitTestTextNearPos(x:Number, y:Number, maxDistance:Number = 0):Number {
            return -1;
        }

        public function setSelectColor(hexColor:uint = 0xFFFF00):void {
        }

        public function setSelected(beginIndex:int, endIndex:int, select:Boolean):void {
        }
    }
}
