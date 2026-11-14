package flash.globalization {
    [API("667")]
    public final class LocaleID {
        public static const DEFAULT:String = "i-default";

        private var _name:String;
        private var _lastOperationStatus:String = LastOperationStatus.NO_ERROR;

        public function LocaleID(name:String) {
            if (name == null) {
                Error.throwError(TypeError, 2007, "name");
            }
            this._name = name;
        }

        public function get lastOperationStatus():String {
            return this._lastOperationStatus;
        }

        public function get name():String {
            return this._name;
        }

        public static function determinePreferredLocales(
            want:Vector.<String>,
            have:Vector.<String>,
            keyword:String = "userinterface"
        ):Vector.<String> {
            if (want == null) {
                Error.throwError(TypeError, 2007, "want");
            }
            if (have == null) {
                Error.throwError(TypeError, 2007, "have");
            }
            var result:Vector.<String> = new Vector.<String>();
            for each (var w:String in want) {
                if (w == null) {
                    continue;
                }
                var normWant:String = w.replace(/_/g, "-").toLowerCase();
                for each (var h:String in have) {
                    if (h != null && h.replace(/_/g, "-").toLowerCase() == normWant) {
                        if (result.indexOf(h) == -1) {
                            result.push(h);
                        }
                    }
                }
                var wantLang:String = normWant.split("-").split("@");
                for each (var h2:String in have) {
                    if (h2 != null) {
                        var haveLang:String = h2.replace(/_/g, "-").toLowerCase().split("-").split("@");
                        if (haveLang == wantLang && result.indexOf(h2) == -1) {
                            result.push(h2);
                        }
                    }
                }
            }
            return result;
        }

        private function basePart():String {
            var idx:int = this._name.indexOf("@");
            return idx >= 0 ? this._name.substring(0, idx) : this._name;
        }

        private function subtags():Array {
            return this.basePart().replace(/_/g, "-").split("-");
        }

        public function getKeysAndValues():Object {
            var obj:Object = new Object();
            var idx:int = this._name.indexOf("@");
            if (idx >= 0) {
                var pairs:Array = this._name.substring(idx + 1).split(";");
                for each (var pair:String in pairs) {
                    var eq:int = pair.indexOf("=");
                    if (eq > 0) {
                        obj[pair.substring(0, eq)] = pair.substring(eq + 1);
                    }
                }
            }
            return obj;
        }

        public function getLanguage():String {
            if (this._name == DEFAULT) {
                return "en";
            }
            var parts:Array = this.subtags();
            return parts.length > 0 ? String(parts).toLowerCase() : "";
        }

        public function getScript():String {
            var parts:Array = this.subtags();
            for (var i:int = 1; i < parts.length; i++) {
                var p:String = String(parts[i]);
                if (p.length == 4 && /^[A-Za-z]{4}$/.test(p)) {
                    return p.charAt(0).toUpperCase() + p.substring(1).toLowerCase();
                }
            }
            return "";
        }

        public function getRegion():String {
            var parts:Array = this.subtags();
            for (var i:int = 1; i < parts.length; i++) {
                var p:String = String(parts[i]);
                if ((p.length == 2 && /^[A-Za-z]{2}$/.test(p)) || (p.length == 3 && /^[0-9]{3}$/.test(p))) {
                    return p.toUpperCase();
                }
            }
            return "";
        }

        public function getVariant():String {
            var parts:Array = this.subtags();
            var variants:Array = [];
            for (var i:int = 1; i < parts.length; i++) {
                var p:String = String(parts[i]);
                if (p.length >= 5 || (p.length == 4 && /^[0-9]/.test(p))) {
                    variants.push(p);
                }
            }
            return variants.join("-");
        }

        public function isRightToLeft():Boolean {
            var lang:String = this.getLanguage();
            return lang == "ar" || lang == "he" || lang == "iw" || lang == "fa" || lang == "ur" || lang == "yi";
        }
    }
}
