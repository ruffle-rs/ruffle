package flash.text.engine {
    public final class TextBlock {
        public var userData;
        
        private var _applyNonLinearFontScaling:Boolean;
        private var _baselineFontDescription:FontDescription = null;
        private var _baselineFontSize:Number = 12;
        private var _baselineZero:String = "roman";
        private var _bidiLevel:int;
        private var _lineRotation:String;
        private var _tabStops:Vector.<TabStop>;
        private var _textJustifier:TextJustifier;
        private var _content:ContentElement;
        
        
        public function TextBlock(content:ContentElement = null,
                                  tabStops:Vector.<TabStop> = null,
                                  textJustifier:TextJustifier = null,
                                  lineRotation:String = "rotate0",
                                  baselineZero:String = "roman",
                                  bidiLevel:int = 0,
                                  applyNonLinearFontScaling:Boolean = true,
                                  baselineFontDescription:FontDescription = null,
                                  baselineFontSize:Number = 12
                                 ) {
            // The order of setting these properties matters- if lineRotation
            // is null/invalid, the rest won't be set because it will throw an error
            if (content) {
                this.content = content;
            }
            if (tabStops) {
                this.tabStops = tabStops;
            }
            if (textJustifier) {
                this.textJustifier = textJustifier;
            } else {
                // This should creaate a new TextJustifier with locale "en", but we don't actually support creating TextJustifiers yet.
            }
            
            this.lineRotation = lineRotation;
            
            if (baselineZero) {
                this.baselineZero = baselineZero;
            }
            if (baselineFontDescription) {
                this.baselineFontDescription = baselineFontDescription;
                this.baselineFontSize = baselineFontSize;
            }
            this.applyNonLinearFontScaling = applyNonLinearFontScaling;
        }
        
        public function get applyNonLinearFontScaling():Boolean {
            return this._applyNonLinearFontScaling;
        }
        
        public function set applyNonLinearFontScaling(value:Boolean):void {
            this._applyNonLinearFontScaling = value;
        }
        
        public function get baselineFontDescription():FontDescription {
            return this._baselineFontDescription;
        }
        
        public function set baselineFontDescription(value:FontDescription):void {
            this._baselineFontDescription = value;
        }
        
        public function get baselineFontSize():Number {
            return this._baselineFontSize;
        }
        
        public function set baselineFontSize(value:Number):void {
            this._baselineFontSize = value;
        }
        
        public function get baselineZero():String {
            return this._baselineZero;
        }
        
        public function set baselineZero(value:String):void {
            this._baselineZero = value;
        }
        
        public function get bidiLevel():int {
            return this._bidiLevel;
        }
        
        public function set bidiLevel(value:int):void {
            this._bidiLevel = value;
        }
        
        public function get lineRotation():String {
            return this._lineRotation;
        }
        
        public function set lineRotation(value:String):void {
            if (value == null) {
                throw new TypeError("Error #2007: Parameter lineRotation must be non-null.", 2007);
            }
            // TODO: This should validate that `value` is a member of TextRotation
            this._lineRotation = value;
        }
        
        // Note: FP returns a copy of the Vector passed to it, so modifying the returned Vector doesn't affect the actual internal representation
        public function get tabStops():Vector.<TabStop> {
            return this._tabStops;
        }
        
        // Note: FP makes a copy of the Vector passed to it, then sets its internal representation to that
        public function set tabStops(value:Vector.<TabStop>):void {
            this._tabStops = value;
        }
        
        public function get textJustifier():TextJustifier {
            return this._textJustifier;
        }
        
        public function set textJustifier(value:TextJustifier):void {
            this._textJustifier = value;
        }
        
        public function get content():ContentElement {
            return this._content;
        }
        
        public function set content(value:ContentElement):void {
            this._content = value;
        }
    }
}
