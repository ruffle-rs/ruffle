package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_method;

    [API("667")]
    public final class CurrencyFormatter {
        private var _requestedLocaleIDName:String;
        private var _decimalSeparator:String = ".";
        private var _digitsType:uint = NationalDigitsType.EUROPEAN;
        private var _fractionalDigits:int = 2;
        private var _groupingPattern:String = "3;*";
        private var _groupingSeparator:String = ",";
        private var _leadingZero:Boolean = true;
        private var _negativeCurrencyFormat:uint = 1;
        private var _negativeSymbol:String = "-";
        private var _positiveCurrencyFormat:uint = 0;
        private var _trailingZeros = true;
        private var _useGrouping = true;

        public function CurrencyFormatter(requestedLocaleIDName:String) {
            stub_constructor("flash.globalization.CurrencyFormatter");
            this._requestedLocaleIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            stub_getter("flash.globalization.CurrencyFormatter", "actualLocaleIDName");
            return "en-US";
        }

        public function get currencyISOCode():String {
            stub_getter("flash.globalization.CurrencyFormatter", "currencyISOCode");
            return "USD";
        }

        public function get currencySymbol():String {
            stub_getter("flash.globalization.CurrencyFormatter", "currencySymbol");
            return "$";
        }

        public function get decimalSeparator():String {
            stub_getter("flash.globalization.CurrencyFormatter", "decimalSeparator");
            return this._decimalSeparator;
        }

        public function set decimalSeparator(value:String):void {
            stub_setter("flash.globalization.CurrencyFormatter", "decimalSeparator");
            this._decimalSeparator = value;
        }

        public function get digitsType():uint {
            stub_getter("flash.globalization.CurrencyFormatter", "digitsType");
            return this._digitsType;
        }

        public function set digitsType(value:uint):void {
            stub_setter("flash.globalization.CurrencyFormatter", "digitsType");
            this._digitsType = value;
        }

        public function get fractionalDigits():int {
            stub_getter("flash.globalization.CurrencyFormatter", "fractionalDigits");
            return this._fractionalDigits;
        }

        public function set fractionalDigits(value:int):void {
            stub_setter("flash.globalization.CurrencyFormatter", "fractionalDigits");
            this._fractionalDigits = value;
        }

        public function get groupingPattern():String {
            stub_getter("flash.globalization.CurrencyFormatter", "groupingPattern");
            return this._groupingPattern;
        }

        public function set groupingPattern(value:String):void {
            stub_setter("flash.globalization.CurrencyFormatter", "groupingPattern");
            this._groupingPattern = value;
        }

        public function get groupingSeparator():String {
            stub_getter("flash.globalization.CurrencyFormatter", "groupingSeparator");
            return this._groupingSeparator;
        }

        public function set groupingSeparator(value:String):void {
            stub_setter("flash.globalization.CurrencyFormatter", "groupingSeparator");
            this._groupingSeparator = value;
        }

        public function get lastOperationStatus():String {
            stub_getter("flash.globalization.CurrencyFormatter", "lastOperationStatus");
            return "noError";
        }

        public function get leadingZero():Boolean {
            stub_getter("flash.globalization.CurrencyFormatter", "leadingZero");
            return this._leadingZero;
        }

        public function set leadingZero(value:Boolean):void {
            stub_setter("flash.globalization.CurrencyFormatter", "leadingZero");
            this._leadingZero = value;
        }

        public function get negativeCurrencyFormat():uint {
            stub_getter("flash.globalization.CurrencyFormatter", "negativeCurrencyFormat");
            return this._negativeCurrencyFormat;
        }

        public function set negativeCurrencyFormat(value:uint):void {
            stub_setter("flash.globalization.CurrencyFormatter", "negativeCurrencyFormat");
            this._negativeCurrencyFormat = value;
        }

        public function get negativeSymbol():String {
            stub_getter("flash.globalization.CurrencyFormatter", "negativeSymbol");
            return this._negativeSymbol;
        }

        public function set negativeSymbol(value:String):void {
            stub_setter("flash.globalization.CurrencyFormatter", "negativeSymbol");
            this._negativeSymbol = value;
        }

        public function get positiveCurrencyFormat():uint {
            stub_getter("flash.globalization.CurrencyFormatter", "positiveCurrencyFormat");
            return this._positiveCurrencyFormat;
        }

        public function set positiveCurrencyFormat(value:uint):void {
            stub_setter("flash.globalization.CurrencyFormatter", "positiveCurrencyFormat");
            this._positiveCurrencyFormat = value;
        }

        public function get requestedLocaleIDName():String {
            return this._requestedLocaleIDName;
        }

        public function get trailingZeros():Boolean {
            stub_getter("flash.globalization.CurrencyFormatter", "trailingZeros");
            return this._trailingZeros;
        }

        public function set trailingZeros(value:Boolean):void {
            stub_setter("flash.globalization.CurrencyFormatter", "trailingZeros");
            this._trailingZeros = value;
        }

        public function get useGrouping():Boolean {
            stub_getter("flash.globalization.CurrencyFormatter", "useGrouping");
            return this._useGrouping;
        }

        public function set useGrouping(value:Boolean):void {
            stub_setter("flash.globalization.CurrencyFormatter", "useGrouping");
            this._useGrouping = value;
        }

        public function format(value:Number, withCurrencySymbol:Boolean = false):String {
            stub_method("flash.globalization.CurrencyFormatter", "format");
            return value.toString();
        }

        public function formattingWithCurrencySymbolIsSafe(requestedISOCode:String):Boolean {
            stub_method("flash.globalization.CurrencyFormatter", "formattingWithCurrencySymbolIsSafe");
            return true;
        }

        public function parse(inputString:String):CurrencyParseResult {
            stub_method("flash.globalization.CurrencyFormatter", "parse");
            return new CurrencyParseResult();
        }

        public function setCurrency(currencyISOCode:String, currencySymbol:String):void {
            stub_method("flash.globalization.CurrencyFormatter", "setCurrency");
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            stub_method("flash.globalization.CurrencyFormatter", "getAvailableLocaleIDNames");
            return new <String>["en-US"];
        }
    }
}
