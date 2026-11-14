package flash.globalization {
    [API("667")]
    public final class StringTools {
        private var _requestedLocaleIDName:String;
        private var _lastOperationStatus:String = LastOperationStatus.NO_ERROR;

        public function StringTools(requestedLocaleIDName:String) {
            if (requestedLocaleIDName == null) {
                Error.throwError(TypeError, 2007, "requestedLocaleIDName");
            }
            this._requestedLocaleIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            return this._requestedLocaleIDName == LocaleID.DEFAULT ? "en-US" : this._requestedLocaleIDName;
        }

        public function get lastOperationStatus():String {
            return this._lastOperationStatus;
        }

        public function get requestedLocaleIDName():String {
            return this._requestedLocaleIDName;
        }

        public function toLowerCase(s:String):String {
            if (s == null) {
                Error.throwError(TypeError, 2007, "s");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return s.toLowerCase();
        }

        public function toUpperCase(s:String):String {
            if (s == null) {
                Error.throwError(TypeError, 2007, "s");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return s.toUpperCase();
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            return new <String>["en-US"];
        }
    }
}
