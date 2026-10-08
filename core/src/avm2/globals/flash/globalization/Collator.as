package flash.globalization {
    [API("667")]
    [Ruffle(InstanceAllocator)]
    public final class Collator {
        public function Collator(requestedLocaleIDName:String, initialMode:String = "sorting") {
            this.init(requestedLocaleIDName, initialMode);
        }

        private native function init(requestedLocaleIDName:String, initialMode:String):void;

        public native function get actualLocaleIDName():String;

        public native function get ignoreCase():Boolean;
        public native function set ignoreCase(value:Boolean):void;

        public native function get ignoreCharacterWidth():Boolean;
        public native function set ignoreCharacterWidth(value:Boolean):void;

        public native function get ignoreDiacritics():Boolean;
        public native function set ignoreDiacritics(value:Boolean):void;

        public native function get ignoreKanaType():Boolean;
        public native function set ignoreKanaType(value:Boolean):void;

        public native function get ignoreSymbols():Boolean;
        public native function set ignoreSymbols(value:Boolean):void;

        public native function get lastOperationStatus():String;

        public native function get numericComparison():Boolean;
        public native function set numericComparison(value:Boolean):void;

        public native function get requestedLocaleIDName():String;

        public native function compare(string1:String, string2:String):int;

        public native function equals(string1:String, string2:String):Boolean;

        public static native function getAvailableLocaleIDNames():Vector.<String>;
    }
}
