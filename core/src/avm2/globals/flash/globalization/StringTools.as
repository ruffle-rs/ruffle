package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [API("667")]
    public final class StringTools {
        private var _requestedLocaleIDName:String;

        public function StringTools(requestedLocaleIDName:String) {
            stub_constructor("flash.globalization.StringTools");
            this._requestedLocaleIDName = requestedLocaleIDName;
        }

        public function get actualLocaleIDName():String {
            stub_getter("flash.globalization.StringTools", "actualLocaleIDName");
            return "en-US";
        }

        public function get lastOperationStatus():String {
            stub_getter("flash.globalization.StringTools", "lastOperationStatus");
            return "noError";
        }

        public function get requestedLocaleIDName():String {
            return this._requestedLocaleIDName;
        }

        public function toLowerCase(s:String):String {
            stub_method("flash.globalization.StringTools", "toLowerCase");
            return s.toLowerCase();
        }

        public function toUpperCase(s:String):String {
            stub_method("flash.globalization.StringTools", "toUpperCase");
            return s.toUpperCase();
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            stub_method("flash.globalization.StringTools", "getAvailableLocaleIDNames");
            return new <String>["en-US"];
        }
    }
}
