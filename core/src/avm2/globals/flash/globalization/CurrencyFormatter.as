package flash.globalization {
    [API("667")]
    public final class CurrencyFormatter {
        private var _requestedLocaleIDName:String;
        private var _currencyISOCode:String = "USD";
        private var _currencySymbol:String = "$";
        private var _decimalSeparator:String = ".";
        private var _digitsType:uint = NationalDigitsType.EUROPEAN;
        private var _fractionalDigits:int = 2;
        private var _groupingPattern:String = "3;*";
        private var _groupingSeparator:String = ",";
        private var _leadingZero:Boolean = true;
        private var _negativeCurrencyFormat:uint = 0;
        private var _negativeSymbol:String = "-";
        private var _positiveCurrencyFormat:uint = 0;
        private var _trailingZeros:Boolean = true;
        private var _useGrouping:Boolean = true;
        private var _lastOperationStatus:String = LastOperationStatus.NO_ERROR;

        public function CurrencyFormatter(requestedLocaleIDName:String) {
            if (requestedLocaleIDName == null) {
                Error.throwError(TypeError, 2007, "requestedLocaleIDName");
            }
            this._requestedLocaleIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            return this._requestedLocaleIDName == LocaleID.DEFAULT ? "en-US" : this._requestedLocaleIDName;
        }

        public function get currencyISOCode():String {
            return this._currencyISOCode;
        }

        public function get currencySymbol():String {
            return this._currencySymbol;
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

        public function get negativeCurrencyFormat():uint {
            return this._negativeCurrencyFormat;
        }
        public function set negativeCurrencyFormat(value:uint):void {
            if (value <= 15) {
                this._negativeCurrencyFormat = value;
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

        public function get positiveCurrencyFormat():uint {
            return this._positiveCurrencyFormat;
        }
        public function set positiveCurrencyFormat(value:uint):void {
            if (value <= 3) {
                this._positiveCurrencyFormat = value;
            }
        }

        public function get requestedLocaleIDName():String {
            return this._requestedLocaleIDName;
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

        public function format(value:Number, withCurrencySymbol:Boolean = false):String {
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            if (isNaN(value)) {
                return "NaN";
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

            var numStr:String = fracPart.length > 0 ? (intPart + this._decimalSeparator + fracPart) : intPart;
            if (numStr.length == 0) {
                numStr = "0";
            }

            var sym:String = withCurrencySymbol ? this._currencySymbol : this._currencyISOCode;
            if (!isNeg) {
                switch (this._positiveCurrencyFormat) {
                    case 1:
                        return numStr + sym;
                    case 2:
                        return sym + " " + numStr;
                    case 3:
                        return numStr + " " + sym;
                    default:
                        return sym + numStr;
                }
            }
            switch (this._negativeCurrencyFormat) {
                case 0:
                    return "(" + sym + numStr + ")";
                case 2:
                    return sym + this._negativeSymbol + numStr;
                case 3:
                    return sym + numStr + this._negativeSymbol;
                case 4:
                    return "(" + numStr + sym + ")";
                case 5:
                    return this._negativeSymbol + numStr + sym;
                default:
                    return this._negativeSymbol + sym + numStr;
            }
        }

        public function formattingWithCurrencySymbolIsSafe(requestedISOCode:String):Boolean {
            if (requestedISOCode == null) {
                Error.throwError(TypeError, 2007, "requestedISOCode");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return requestedISOCode.toUpperCase() == this._currencyISOCode.toUpperCase();
        }

        public function parse(inputString:String):CurrencyParseResult {
            if (inputString == null) {
                Error.throwError(TypeError, 2007, "inputString");
            }
            var cleaned:String = inputString;
            var detectedSymbol:String = "";
            if (this._currencySymbol.length > 0 && cleaned.indexOf(this._currencySymbol) >= 0) {
                detectedSymbol = this._currencySymbol;
                cleaned = cleaned.split(this._currencySymbol).join("");
            } else if (this._currencyISOCode.length > 0 && cleaned.indexOf(this._currencyISOCode) >= 0) {
                detectedSymbol = this._currencyISOCode;
                cleaned = cleaned.split(this._currencyISOCode).join("");
            }
            cleaned = cleaned.replace(/^\s+|\s+$/g, "");
            var isNeg:Boolean = false;
            if (cleaned.charAt(0) == "(" && cleaned.charAt(cleaned.length - 1) == ")") {
                isNeg = true;
                cleaned = cleaned.substring(1, cleaned.length - 1);
            } else if (this._negativeSymbol.length > 0 && cleaned.indexOf(this._negativeSymbol) >= 0) {
                isNeg = true;
                cleaned = cleaned.split(this._negativeSymbol).join("");
            }
            if (this._groupingSeparator.length > 0) {
                cleaned = cleaned.split(this._groupingSeparator).join("");
            }
            if (this._decimalSeparator.length > 0 && this._decimalSeparator != ".") {
                cleaned = cleaned.split(this._decimalSeparator).join(".");
            }
            var num:Number = Number(cleaned);
            if (isNaN(num)) {
                this._lastOperationStatus = LastOperationStatus.PARSE_ERROR;
                return new CurrencyParseResult(NaN, "");
            }
            if (isNeg) {
                num = -num;
            }
                this._lastOperationStatus = LastOperationStatus.NO_ERROR;
                return new CurrencyParseResult(num, detectedSymbol);
            }
        public function setCurrency(currencyISOCode:String, currencySymbol:String):void {
            if (currencyISOCode == null) {
                Error.throwError(TypeError, 2007, "currencyISOCode");
            }
            if (currencySymbol == null) {
                Error.throwError(TypeError, 2007, "currencySymbol");
            }
            this._currencyISOCode = currencyISOCode;
            this._currencySymbol = currencySymbol;
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            }
        public static function getAvailableLocaleIDNames():Vector.<String> {
            return new ["en-US"];
            }
    }
}
