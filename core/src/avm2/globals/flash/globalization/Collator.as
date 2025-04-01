package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [API("667")]
    public final class Collator {
        private var _requestedLocaleIDName:String;

        public function Collator(requestedLocaleIDName:String, initialMode:String = "sorting") {
            stub_constructor("flash.globalization.Collator");
            this._requestedLocaleIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            stub_getter("flash.globalization.Collator", "actualLocaleIDName");
            return "en-US";
        }

        public function get ignoreCase():Boolean {
            stub_getter("flash.globalization.Collator", "ignoreCase");
            return false;
        }

        public function set ignoreCase(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "ignoreCase");
        }

        public function get ignoreCharacterWidth():Boolean {
            stub_getter("flash.globalization.Collator", "ignoreCharacterWidth");
            return false;
        }

        public function set ignoreCharacterWidth(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "ignoreCharacterWidth");
        }

        public function get ignoreDiacritics():Boolean {
            stub_getter("flash.globalization.Collator", "ignoreDiacritics");
            return false;
        }

        public function set ignoreDiacritics(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "ignoreDiacritics");
        }

        public function get ignoreKanaType():Boolean {
            stub_getter("flash.globalization.Collator", "ignoreKanaType");
            return false;
        }

        public function set ignoreKanaType(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "ignoreKanaType");
        }

        public function get ignoreSymbols():Boolean {
            stub_getter("flash.globalization.Collator", "ignoreSymbols");
            return false;
        }

        public function set ignoreSymbols(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "ignoreSymbols");
        }

        public function get lastOperationStatus():String {
            stub_getter("flash.globalization.Collator", "lastOperationStatus");
            return "noError";
        }

        public function get numericComparison():Boolean {
            stub_getter("flash.globalization.Collator", "numericComparison");
            return false;
        }

        public function set numericComparison(value:Boolean):void {
            stub_setter("flash.globalization.Collator", "numericComparison");
        }

        public function get requestedLocaleIDName():String {
            return this._requestedLocaleIDName;
        }

        public function compare(string1:String, string2:String):int {
            stub_method("flash.globalization.Collator", "compare");
            return string1.localeCompare(string2);
        }

        public function equals(string1:String, string2:String):Boolean {
            stub_method("flash.globalization.Collator", "equals");
            return this.compare(string1, string2) == 0;
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            stub_method("flash.globalization.Collator", "getAvailableLocaleIDNames");
            return new <String>["en-US"];
        }
    }
}
