package flash.text.engine {
    import __ruffle__.stub_method;

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
            if (value == null) throwNonNull("fontName");

            this._fontName = value;
        }

        public function get fontWeight():String {
            return this._fontWeight;
        }

        public function set fontWeight(value:String):void {
            if (value == null) throwNonNull("fontWeight");
            if (value != FontWeight.NORMAL && value != FontWeight.BOLD) {
                throwNotAccepted("fontWeight");
            }

            this._fontWeight = value;
        }

        public function get fontPosture():String {
            return this._fontPosture;
        }

        public function set fontPosture(value:String):void {
            if (value == null) throwNonNull("fontPosture");
            if (value != FontPosture.NORMAL && value != FontPosture.ITALIC) {
                throwNotAccepted("fontPosture");
            }

            this._fontPosture = value;
        }

        public function get fontLookup():String {
            return this._fontLookup;
        }

        public function set fontLookup(value:String):void {
            if (value == null) throwNonNull("fontLookup");
            if (value != FontLookup.DEVICE && value != FontLookup.EMBEDDED_CFF) {
                throwNotAccepted("fontLookup");
            }

            this._fontLookup = value;
        }

        public function get renderingMode():String {
            return this._renderingMode;
        }

        public function set renderingMode(value:String):void {
            if (value == null) throwNonNull("renderingMode");
            if (value != RenderingMode.NORMAL && value != RenderingMode.CFF) {
                throwNotAccepted("renderingMode");
            }

            this._renderingMode = value;
        }

        public function get cffHinting():String {
            return this._cffHinting;
        }

        public function set cffHinting(value:String):void {
            if (value == null) throwNonNull("cffHinting");
            if (value != CFFHinting.NONE && value != CFFHinting.HORIZONTAL_STEM) {
                throwNotAccepted("cffHinting");
            }

            this._cffHinting = value;
        }

        public static function isFontCompatible(fontName: String, fontWeight: String, fontPosture: String): Boolean {
            stub_method("flash.text.engine.FontDescription", "isFontCompatible");
            return false;
        }

        private static function throwNonNull(name: String) {
            throw new TypeError("Error #2007: Parameter " + name + " must be non-null.", 2007);
        }

        private static function throwNotAccepted(name: String) {
            throw new ArgumentError("Error #2008: Parameter " + name + " must be one of the accepted values.", 2008);
        }
    }
}
