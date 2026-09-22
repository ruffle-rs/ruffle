package flash.text.engine {
    import __ruffle__.stub_setter;

    import flash.utils.getQualifiedClassName;

    [API("662")]
    public class TextJustifier {
        private var _lineJustification:String = null;
        private var _locale:String = null;

        public function TextJustifier(locale:String, lineJustification:String) {
            // TODO: Is there a better way to do this?
            if (getQualifiedClassName(this) === "flash.text.engine::TextJustifier") {
                throw new ArgumentError("Error #2012: TextJustifier$ class cannot be instantiated.", 2012);
            }

            this.setLocale(locale);
            this.lineJustification = lineJustification;
        }

        private function setLocale(locale:String):void {
            if (locale == null) {
                throw new TypeError("Error #2007: Parameter locale must be non-null.", 2007);
            }

            if (locale.length < 2) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            this._locale = locale;
        }

        public function get lineJustification():String {
            return this._lineJustification;
        }

        public function set lineJustification(value:String):void {
            this._lineJustification = value;
        }

        public function get locale():String {
            return this._locale;
        }

        public function clone():TextJustifier {
            return null;
        }
    }
}
