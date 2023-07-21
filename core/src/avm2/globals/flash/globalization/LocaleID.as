package flash.globalization {
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;
    import flash.globalization.LastOperationStatus;

    public final class LocaleID {
        public static const DEFAULT:String = "i-default";

        private var _name:String;

        public function LocaleID(name:String) {
            stub_constructor("flash.globalization.LocaleID");
            this._name = name;
        }

        public function get lastOperationStatus():String {
            stub_getter("flash.globalization.LocaleID", "lastOperationStatus");
            return LastOperationStatus.NO_ERROR;
        }

        public function get name():String {
            return this._name;
        }

        public static function determinePreferredLocales(want:Vector.<String>, have:Vector.<String>, keyword:String = "userinterface"):Vector.<String> {
            stub_method("flash.globalization.LocaleID", "determinePreferredLocales");
            return null;
        }

        public function getKeysAndValues():Object {
            stub_method("flash.globalization.LocaleID", "getKeysAndValues");
            return new Object();
        }

        public function getLanguage():String {
            stub_method("flash.globalization.LocaleID", "getLanguage");
            return this._name;
        }

        public function getRegion():String {
            stub_method("flash.globalization.LocaleID", "getRegion");
            return "";
        }

        public function getScript():String {
            stub_method("flash.globalization.LocaleID", "getScript");
            return "";
        }

        public function getVariant():String {
            stub_method("flash.globalization.LocaleID", "getVariant");
            return "";
        }

        public function isRightToLeft():Boolean {
            stub_method("flash.globalization.LocaleID", "isRightToLeft");
            return false;
        }
    }
}
