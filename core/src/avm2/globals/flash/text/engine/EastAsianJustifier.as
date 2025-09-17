package flash.text.engine {
    [API("662")]
    public final class EastAsianJustifier extends TextJustifier {
        private var _justificationStyle:String;
        private var _composeTrailingIdeographicSpaces:Boolean;

        public function EastAsianJustifier(locale:String = "ja", lineJustification:String = "allButLast", justificationStyle:String = "pushInKinsoku") {
            super(locale, lineJustification);
            this._justificationStyle = justificationStyle;
        }

        public function get justificationStyle():String {
            return this._justificationStyle;
        }

        public function set justificationStyle(value:String):void {
            // TODO: Validate the argument
            this._justificationStyle = value;
        }

        [API("674")]
        public function get composeTrailingIdeographicSpaces():Boolean {
            return this._composeTrailingIdeographicSpaces;
        }

        [API("674")]
        public function set composeTrailingIdeographicSpaces(value:Boolean):void {
            this._composeTrailingIdeographicSpaces = value;
        }

        override public function clone():TextJustifier {
            var copy = new EastAsianJustifier(this.locale, this.lineJustification, this.justificationStyle);
            copy.composeTrailingIdeographicSpaces = this.composeTrailingIdeographicSpaces;
            return copy;
        }
    }
}
