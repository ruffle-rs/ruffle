package flash.globalization {
    [API("667")]
    public final class CurrencyParseResult {
        private var _value:Number;
        private var _currencyString:String;

        public function CurrencyParseResult(value:Number = NaN, symbol:String = "") {
            this._value = value;
            this._currencyString = symbol;
        }

        public function get value():Number {
            return this._value;
        }

        public function get currencyString():String {
            return this._currencyString;
        }
    }
}
