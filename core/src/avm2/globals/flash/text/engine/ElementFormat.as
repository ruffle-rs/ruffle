package flash.text.engine {
    import __ruffle__.stub_method;

    import flash.geom.Rectangle;

    [API("662")]
    public final class ElementFormat {
        private var _alignmentBaseline:String;

        private var _alpha:Number;

        private var _baselineShift:Number;

        private var _breakOpportunity:String;

        [Ruffle(NativeAccessible)]
        private var _color:uint;

        private var _digitCase:String;

        private var _digitWidth:String;

        private var _dominantBaseline:String;

        [Ruffle(NativeAccessible)]
        private var _fontDescription:FontDescription;

        [Ruffle(NativeAccessible)]
        private var _fontSize:Number;

        private var _kerning:String;

        private var _ligatureLevel:String;

        private var _locale:String;

        private var _textRotation:String;

        private var _trackingLeft:Number;

        private var _trackingRight:Number;

        private var _typographicCase:String;


        public function ElementFormat(
            fontDescription:FontDescription = null, fontSize:Number = 12, color:uint = 0, alpha:Number = 1,
            textRotation:String = "auto", dominantBaseline:String = "roman",
            alignmentBaseline:String = "useDominantBaseline", baselineShift:Number = 0, kerning:String = "on",
            trackingRight:Number = 0, trackingLeft:Number = 0, locale:String = "en", breakOpportunity:String = "auto",
            digitCase:String = "default", digitWidth:String = "default", ligatureLevel:String = "common",
            typographicCase:String = "default"
        ) {
            this.fontDescription = (fontDescription != null) ? fontDescription : new FontDescription();

            this.alignmentBaseline = alignmentBaseline;
            this.alpha = alpha;
            this.baselineShift = baselineShift;
            this.breakOpportunity = breakOpportunity;
            this.color = color;
            this.digitCase = digitCase;
            this.digitWidth = digitWidth;
            this.dominantBaseline = dominantBaseline;
            this.fontSize = fontSize;
            this.kerning = kerning;
            this.ligatureLevel = ligatureLevel;
            this.locale = locale;
            this.textRotation = textRotation;
            this.trackingLeft = trackingLeft;
            this.trackingRight = trackingRight;
            this.typographicCase = typographicCase;
        }

        public function get alignmentBaseline():String {
            return this._alignmentBaseline;
        }

        public function set alignmentBaseline(value:String):void {
            this._alignmentBaseline = value;
        }

        public function get alpha():Number {
            return this._alpha;
        }

        public function set alpha(value:Number):void {
            this._alpha = value;
        }

        public function get baselineShift():Number {
            return this._baselineShift;
        }

        public function set baselineShift(value:Number):void {
            this._baselineShift = value;
        }

        public function get breakOpportunity():String {
            return this._breakOpportunity;
        }

        public function set breakOpportunity(value:String):void {
            this._breakOpportunity = value;
        }

        public function get color():uint {
            return this._color;
        }

        public function set color(value:uint):void {
            this._color = value;
        }

        public function get digitCase():String {
            return this._digitCase;
        }

        public function set digitCase(value:String):void {
            this._digitCase = value;
        }

        public function get digitWidth():String {
            return this._digitWidth;
        }

        public function set digitWidth(value:String):void {
            this._digitWidth = value;
        }

        public function get dominantBaseline():String {
            return this._dominantBaseline;
        }

        public function set dominantBaseline(value:String):void {
            this._dominantBaseline = value;
        }

        public function get fontDescription():FontDescription {
            return this._fontDescription;
        }

        public function set fontDescription(value:FontDescription):void {
            this._fontDescription = value;
        }

        public function get fontSize():Number {
            return this._fontSize;
        }

        public function set fontSize(value:Number):void {
            this._fontSize = value;
        }

        public function get kerning():String {
            return this._kerning;
        }

        public function set kerning(value:String):void {
            this._kerning = value;
        }

        public function get ligatureLevel():String {
            return this._ligatureLevel;
        }

        public function set ligatureLevel(value:String):void {
            this._ligatureLevel = value;
        }

        public function get locale():String {
            return this._locale;
        }

        public function set locale(value:String):void {
            this._locale = value;
        }

        public function get textRotation():String {
            return this._textRotation;
        }

        public function set textRotation(value:String):void {
            this._textRotation = value;
        }

        public function get trackingLeft():Number {
            return this._trackingLeft;
        }

        public function set trackingLeft(value:Number):void {
            this._trackingLeft = value;
        }

        public function get trackingRight():Number {
            return this._trackingRight;
        }

        public function set trackingRight(value:Number):void {
            this._trackingRight = value;
        }

        public function get typographicCase():String {
            return this._typographicCase;
        }

        public function set typographicCase(value:String):void {
            this._typographicCase = value;
        }

        public function clone():ElementFormat {
            var fd = this.fontDescription ? this.fontDescription.clone() : null;
            return new ElementFormat(
                fd, this.fontSize, this.color, this.alpha, this.textRotation,
                this.dominantBaseline, this.alignmentBaseline, this.baselineShift, this.kerning,
                this.trackingRight, this.trackingLeft, this.locale, this.breakOpportunity,
                this.digitCase, this.digitWidth, this.ligatureLevel, this.typographicCase
            );
        }


        public function getFontMetrics():FontMetrics {
            stub_method("flash.text.engine.ElementFormat", "getFontMetrics");
            var emBox:Rectangle = new Rectangle(0, _fontSize * -0.8, _fontSize, _fontSize);
            return new FontMetrics(
                emBox, -5, 1.2, 1.8, 1.2, 0.075, 0.6, -0.35, 0.6, 0.0
            );
        }
    }
}
