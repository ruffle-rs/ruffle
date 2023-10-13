package flash.text.engine {
    import flash.utils.getQualifiedClassName;

    public class TextJustifier {
        private var _lineJustification:String = null;
        public function TextJustifier(locale:String, lineJustification:String) {
            // TODO: Is there a better way to do this?
            if (getQualifiedClassName(this) === "flash.text.engine::TextJustifier") {
                throw new ArgumentError("Error #2012: TextJustifier$ class cannot be instantiated.", 2012);
            }

            this.lineJustification = lineJustification;
        }

        public function get lineJustification():String {
            return this._lineJustification;
        }

        public function set lineJustification(value:String):void {
            this._lineJustification = value;
        }

        public function clone():TextJustifier {
            return null;
        }
    }
}
