package flash.globalization {
    public final class NumberParseResult {
        private var _endIndex:int;
        private var _startIndex:int;
        private var _value:Number;

        public function NumberParseResult(value:Number = NaN, startIndex:int = 0x7fffffff, endIndex:int = 0x7fffffff) {
            this._value = value;
            this._startIndex = startIndex;
            this._endIndex = endIndex;
        }

        public function get endIndex():int {
            return this._endIndex;
        }

        public function get startIndex():int {
            return this._startIndex;
        }

        public function get value():Number {
            return this._value;
        }
    }
}
