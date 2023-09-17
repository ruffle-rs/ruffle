package flash.text.engine {
    public final class FontDescription {
        private var _fontName:String;

        private var _fontWeight:String;

        private var _fontPosture:String;

        private var _fontLookup:String;

        private var _renderingMode:String;

        private var _cffHinting:String;



        public function FontDescription(fontName:String = "_serif", fontWeight:String = "normal", fontPosture:String = "normal",
                                        fontLookup:String = "device", renderingMode:String = "cff", cffHinting:String = "horizontalStem") {
            this.fontName = fontName;
            this.fontWeight = fontWeight;
            this.fontPosture = fontPosture;
            this.fontLookup = fontLookup;
            this.renderingMode = renderingMode;
            this.cffHinting = cffHinting;
        }

        public function get fontName():String {
            return this._fontName;
        }

        public function set fontName(value:String):void {
            this._fontName = value;
        }

        public function get fontWeight():String {
            return this._fontWeight;
        }

        public function set fontWeight(value:String):void {
            this._fontWeight = value;
        }

        public function get fontPosture():String {
            return this._fontPosture;
        }

        public function set fontPosture(value:String):void {
            this._fontPosture = value;
        }

        public function get fontLookup():String {
            return this._fontLookup;
        }

        public function set fontLookup(value:String):void {
            this._fontLookup = value;
        }

        public function get renderingMode():String {
            return this._renderingMode;
        }

        public function set renderingMode(value:String):void {
            this._renderingMode = value;
        }

        public function get cffHinting():String {
            return this._cffHinting;
        }

        public function set cffHinting(value:String):void {
            this._cffHinting = value;
        }
    }
}
