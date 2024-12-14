package flash.text {

    import __ruffle__.stub_constructor;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    public class TextSnapshot {
        public function TextSnapshot() {
            stub_constructor("flash.text.TextSnapshot");
        }

        public function get charCount():int {
            stub_getter("flash.text.TextSnapshot", "charCount");
            return 0;
        }

        public function findText(beginIndex:int, textToFind:String, caseSensitive:Boolean):int {
            stub_method("flash.text.TextSnapshot", "findText");
            return -1;
        }

        public function getSelected(beginIndex:int, endIndex:int):Boolean {
            stub_method("flash.text.TextSnapshot", "getSelected");
            return false;
        }

        public function getSelectedText(includeLineEndings:Boolean = false):String {
            stub_method("flash.text.TextSnapshot", "getSelectedText");
            return "";
        }

        public function getText(beginIndex:int, endIndex:int, includeLineEndings:Boolean = false):String {
            stub_method("flash.text.TextSnapshot", "getText");
            return "";
        }

        public function getTextRunInfo(beginIndex:int, endIndex:int):Array {
            stub_method("flash.text.TextSnapshot", "getTextRunInfo");
            return [];
        }

        public function hitTestTextNearPos(x:Number, y:Number, maxDistance:Number = 0):Number {
            stub_method("flash.text.TextSnapshot", "hitTestTextNearPos");
            return -1;
        }

        public function setSelectColor(hexColor:uint = 0xFFFF00):void {
            stub_method("flash.text.TextSnapshot", "setSelectColor");
        }

        public function setSelected(beginIndex:int, endIndex:int, select:Boolean):void {
            stub_method("flash.text.TextSnapshot", "setSelected");
        }
    }
}
