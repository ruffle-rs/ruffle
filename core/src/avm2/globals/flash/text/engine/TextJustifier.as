package flash.text.engine {
    import flash.utils.getQualifiedClassName;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;


    [API("662")]
    public class TextJustifier {
        private var _lineJustification:String = null;
        private var _locale:String = "en";
        public function TextJustifier(locale:String, lineJustification:String) {
            // TODO: Is there a better way to do this?
            if (getQualifiedClassName(this) === "flash.text.engine::TextJustifier") {
                throw new ArgumentError("Error #2012: TextJustifier$ class cannot be instantiated.", 2012);
            }

            // TODO: Validate locale
            this._locale = locale;
            this.lineJustification = lineJustification;
        }

        public function get lineJustification():String {
            return this._lineJustification;
        }

        public function set lineJustification(value:String):void {
            this._lineJustification = value;
        }

        public function get locale():String {
            stub_getter("flash.text.engine.TextJustifier", "locale");
            return this._locale;
        }

        public function clone():TextJustifier {
            return null;
        }
    }
}
