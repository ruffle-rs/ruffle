package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import flash.globalization.LastOperationStatus;
    import flash.globalization.NationalDigitsType;
    import flash.globalization.NumberParseResult;

    public final class NumberFormatter {
        private var _decimalSeparator = ".";
        private var _digitsType = NationalDigitsType.EUROPEAN;
        private var _fractionalDigits = 2;
        private var _groupingPattern = "3;*";
        private var _groupingSeparator = ",";
        private var _leadingZero = true;
        private var _localeIDName:String;
        private var _negativeNumberFormat = 1;
        private var _negativeSymbol = "-";
        private var _trailingZeros = true;
        private var _useGrouping = true;

        public function NumberFormatter(requestedLocaleIDName:String) {
            stub_constructor("flash.globalization.NumberFormatter");
            this._localeIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            stub_getter("flash.globalization.NumberFormatter", "actualLocaleIDName");
            return this._localeIDName;
        }

        public function get decimalSeparator():String {
            return this._decimalSeparator;
        }
        public function set decimalSeparator(value:String):void {
            this._decimalSeparator = value;
        }

        public function get digitsType():uint {
            return this._digitsType;
        }
        public function set digitsType(value:uint):void {
            this._digitsType = value;
        }

        public function get fractionalDigits():int {
            return this._fractionalDigits;
        }
        public function set fractionalDigits(value:int):void {
            this._fractionalDigits = value;
        }

        public function get groupingPattern():String {
            return this._groupingPattern;
        }
        public function set groupingPattern(value:String):void {
            this._groupingPattern = value;
        }

        public function get groupingSeparator():String {
            return this._groupingSeparator;
        }
        public function set groupingSeparator(value:String):void {
            this._groupingSeparator = value;
        }

        public function get lastOperationStatus():String {
            stub_getter("flash.globalization.NumberFormatter", "lastOperationStatus");
            return LastOperationStatus.NO_ERROR;
        }

        public function get leadingZero():Boolean {
            return this._leadingZero;
        }
        public function set leadingZero(value:Boolean):void {
            this._leadingZero = value;
        }

        public function get negativeNumberFormat():uint {
            return this._negativeNumberFormat;
        }
        public function set negativeNumberFormat(value:uint):void {
            if (value >= 0 && value <= 4) {
                this._negativeNumberFormat = value;
            }
        }

        public function get negativeSymbol():String {
            return this._negativeSymbol;
        }
        public function set negativeSymbol(value:String):void {
            this._negativeSymbol = value;
        }

        public function get requestedLocaleIDName():String {
            return this._localeIDName;
        }

        public function get trailingZeros():Boolean {
            return this._trailingZeros;
        }
        public function set trailingZeros(value:Boolean):void {
            this._trailingZeros = value;
        }

        public function get useGrouping():Boolean {
            return this._useGrouping;
        }
        public function set useGrouping(value:Boolean):void {
            this._useGrouping = value;
        }

        public function formatInt(value:int):String {
            stub_method("flash.globalization.NumberFormatter", "formatInt");
            return value.toString();
        }

        public function formatNumber(value:Number):String {
            stub_method("flash.globalization.NumberFormatter", "formatNumber");
            return value.toString();
        }

        public function formatUint(value:uint):String {
            stub_method("flash.globalization.NumberFormatter", "formatUint");
            return value.toString();
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            stub_method("flash.globalization.NumberFormatter", "getAvailableLocaleIDNames");
            return new <String>["en-US"];
        }

        public function parse(parseString:String):NumberParseResult {
            stub_method("flash.globalization.NumberFormatter", "parse");
            return new NumberParseResult();
        }

        public function parseNumber(parseString:String):Number {
            stub_method("flash.globalization.NumberFormatter", "parseNumber");
            return Number(parseString);
        }
    }
}
