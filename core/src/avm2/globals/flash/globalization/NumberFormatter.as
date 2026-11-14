package flash.globalization {
    [API("667")]
    public final class NumberFormatter {
        private var _decimalSeparator:String = ".";
        private var _digitsType:uint = NationalDigitsType.EUROPEAN;
        private var _fractionalDigits:int = 2;
        private var _groupingPattern:String = "3;*";
        private var _groupingSeparator:String = ",";
        private var _leadingZero:Boolean = true;
        private var _localeIDName:String;
        private var _negativeNumberFormat:uint = 1;
        private var _negativeSymbol:String = "-";
        private var _trailingZeros:Boolean = true;
        private var _useGrouping:Boolean = true;
        private var _lastOperationStatus:String = LastOperationStatus.NO_ERROR;

        public function NumberFormatter(requestedLocaleIDName:String) {
            if (requestedLocaleIDName == null) {
                Error.throwError(TypeError, 2007, "requestedLocaleIDName");
            }
            this._localeIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            return this._localeIDName == LocaleID.DEFAULT ? "en-US" : this._localeIDName;
        }

        public function get decimalSeparator():String {
            return this._decimalSeparator;
        }
        public function set decimalSeparator(value:String):void {
            if (value == null) {
                Error.throwError(TypeError, 2007, "value");
            }
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
            if (value == null) {
                Error.throwError(TypeError, 2007, "value");
            }
            this._groupingPattern = value;
        }

        public function get groupingSeparator():String {
            return this._groupingSeparator;
        }
        public function set groupingSeparator(value:String):void {
            if (value == null) {
                Error.throwError(TypeError, 2007, "value");
            }
            this._groupingSeparator = value;
        }

        public function get lastOperationStatus():String {
            return this._lastOperationStatus;
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
            if (value <= 4) {
                this._negativeNumberFormat = value;
            }
        }

        public function get negativeSymbol():String {
            return this._negativeSymbol;
        }
        public function set negativeSymbol(value:String):void {
            if (value == null) {
                Error.throwError(TypeError, 2007, "value");
            }
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

        private function applyGrouping(intStr:String):String {
            if (!this._useGrouping || this._groupingSeparator.length == 0 || intStr.length <= 3) {
                return intStr;
            }
            var parts:Array = [];
            var i:int = intStr.length;
            while (i > 3) {
                parts.unshift(intStr.substring(i - 3, i));
                i -= 3;
            }
            if (i > 0) {
                parts.unshift(intStr.substring(0, i));
            }
            return parts.join(this._groupingSeparator);
        }

        private function applyNegative(formatted:String, isNeg:Boolean):String {
            if (!isNeg) {
                return formatted;
            }
            switch (this._negativeNumberFormat) {
                case 0:
                    return "(" + formatted + ")";
                case 2:
                    return this._negativeSymbol + " " + formatted;
                case 3:
                    return formatted + this._negativeSymbol;
                case 4:
                    return formatted + " " + this._negativeSymbol;
                default:
                    return this._negativeSymbol + formatted;
            }
        }

        public function formatInt(value:int):String {
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            var isNeg:Boolean = value < 0;
            var absVal:Number = Math.abs(Number(value));
            var intStr:String = this.applyGrouping(Math.floor(absVal).toString());
            return this.applyNegative(intStr, isNeg);
        }

        public function formatUint(value:uint):String {
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return this.applyGrouping(value.toString());
        }

        public function formatNumber(value:Number):String {
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            if (isNaN(value)) {
                return "NaN";
            }
            if (!isFinite(value)) {
                return value < 0 ? this._negativeSymbol + "Infinity" : "Infinity";
            }
            var isNeg:Boolean = value < 0;
            var absVal:Number = Math.abs(value);
            var fracDigits:int = this._fractionalDigits < 0 ? 0 : this._fractionalDigits;
            var fixed:String = absVal.toFixed(fracDigits);
            var dotIdx:int = fixed.indexOf(".");
            var intPart:String = dotIdx >= 0 ? fixed.substring(0, dotIdx) : fixed;
            var fracPart:String = dotIdx >= 0 ? fixed.substring(dotIdx + 1) : "";

            if (!this._leadingZero && intPart == "0" && fracPart.length > 0) {
                intPart = "";
            } else {
                intPart = this.applyGrouping(intPart);
            }

            if (!this._trailingZeros && fracPart.length > 0) {
                fracPart = fracPart.replace(/0+$/, "");
            }

            var combined:String = fracPart.length > 0 ? (intPart + this._decimalSeparator + fracPart) : intPart;
            if (combined.length == 0) {
                combined = "0";
            }
            return this.applyNegative(combined, isNeg);
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            return new <String>["en-US"];
        }

        public function parse(parseString:String):NumberParseResult {
            if (parseString == null) {
                Error.throwError(TypeError, 2007, "parseString");
            }
            var startIndex:int = 0;
            while (startIndex < parseString.length && /\s/.test(parseString.charAt(startIndex))) {
                startIndex++;
            }
            var cleaned:String = parseString.substring(startIndex);
            var isNeg:Boolean = false;
            if (cleaned.charAt(0) == "(" && cleaned.indexOf(")") > 0) {
                isNeg = true;
                cleaned = cleaned.substring(1, cleaned.indexOf(")"));
            } else if (this._negativeSymbol.length > 0 && cleaned.indexOf(this._negativeSymbol) == 0) {
                isNeg = true;
                cleaned = cleaned.substring(this._negativeSymbol.length);
            }
            if (this._groupingSeparator.length > 0) {
                cleaned = cleaned.split(this._groupingSeparator).join("");
            }
            if (this._decimalSeparator.length > 0 && this._decimalSeparator != ".") {
                cleaned = cleaned.split(this._decimalSeparator).join(".");
            }
            var match:Array = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/.exec(cleaned);
            if (match == null) {
                this._lastOperationStatus = LastOperationStatus.PARSE_ERROR;
                return new NumberParseResult(NaN, int.MAX_VALUE, int.MAX_VALUE);
            }
            var num:Number = Number(match);
            if (isNeg) {
                num = -num;
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return new NumberParseResult(num, startIndex, parseString.length);
        }

        public function parseNumber(parseString:String):Number {
            return this.parse(parseString).value;
        }
    }
}
