package flash.text.engine {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [API("662")]
    public final class TextBlock {
        public var userData;

        private var _applyNonLinearFontScaling:Boolean;
        private var _baselineFontDescription:FontDescription = null;
        private var _baselineFontSize:Number = 12;
        private var _baselineZero:String = "roman";
        [Ruffle(NativeAccessible)]
        private var _bidiLevel:int;
        private var _lineRotation:String;
        private var _tabStops:Vector.<TabStop>;
        private var _textJustifier:TextJustifier;

        [Ruffle(NativeAccessible)]
        private var _content:ContentElement;

        [Ruffle(NativeAccessible)]
        private var _textLineCreationResult:String = null;

        [Ruffle(NativeAccessible)]
        private var _firstLine:TextLine = null;

        [Ruffle(NativeAccessible)]
        private var _lastLine:TextLine = null;

        public function TextBlock(
            content:ContentElement = null,
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
                // This should create a new TextJustifier with locale "en", but we don't actually support creating TextJustifiers yet.
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

        public native function createTextLine(
            previousLine:TextLine = null,
            width:Number = 1000000,
            lineOffset:Number = 0,
            fitSomething:Boolean = false
        ):TextLine;

        public native function recreateTextLine(
            textLine:TextLine,
            previousLine:TextLine = null,
            width:Number = 1000000,
            lineOffset:Number = 0,
            fitSomething:Boolean = false
        ):TextLine;

        public function get textLineCreationResult():String {
            return this._textLineCreationResult;
        }

        public function get firstLine():TextLine {
            return this._firstLine;
        }

        public function get lastLine():TextLine {
            return this._lastLine;
        }

        public function releaseLines(start:TextLine, end:TextLine):void {
            if (!start || !end) {
                return;
            }
            var beforeStart:TextLine = start._previousLine;
            var afterEnd:TextLine = end._nextLine;
            var node:TextLine = start;
            while (node) {
                var next:TextLine = node._nextLine;
                node._validity = TextLineValidity.INVALID;
                node._textBlock = null;
                node._previousLine = null;
                node._nextLine = null;
                if (node == end) {
                    break;
                }
                node = next;
            }

            if (beforeStart) {
                beforeStart._nextLine = afterEnd;
            } else {
                this._firstLine = afterEnd;
            }
            if (afterEnd) {
                afterEnd._previousLine = beforeStart;
            } else {
                this._lastLine = beforeStart;
            }
        }

        public function get firstInvalidLine():TextLine {
            var line:TextLine = this._firstLine;
            while (line != null) {
                if (line.validity != TextLineValidity.VALID) {
                    return line;
                }
                line = line._nextLine;
            }
            return null;
        }

        public function getTextLineAtCharIndex(charIndex:int):TextLine {
            var line:TextLine = this._firstLine;
            while (line != null) {
                var start:int = line.textBlockBeginIndex;
                var end:int = start + line.rawTextLength;
                if (charIndex >= start && charIndex < end) {
                    return line;
                }
                line = line._nextLine;
            }
            return null;
        }

        public function findNextAtomBoundary(charPos:int):int {
            var content:ContentElement = this._content;
            if (content == null || content.text == null) {
                return charPos;
            }
            var next:int = charPos + 1;
            if (next > content.text.length) {
                next = content.text.length;
            }
            return next;
        }

        public function findPreviousAtomBoundary(charPos:int):int {
            var prev:int = charPos - 1;
            return prev < 0 ? 0 : prev;
        }

        public function findNextWordBoundary(charPos:int):int {
            var content:ContentElement = this._content;
            if (content == null || content.text == null) {
                return charPos;
            }
            var text:String = content.text;
            var i:int = charPos;
            while (i < text.length && !_isWhitespace(text.charCodeAt(i))) {
                i++;
            }
            while (i < text.length && _isWhitespace(text.charCodeAt(i))) {
                i++;
            }
            return i;
        }

        public function findPreviousWordBoundary(charPos:int):int {
            var content:ContentElement = this._content;
            if (content == null || content.text == null) {
                return charPos;
            }
            var text:String = content.text;
            var i:int = charPos - 1;
            while (i > 0 && _isWhitespace(text.charCodeAt(i))) {
                i--;
            }
            while (i > 0 && !_isWhitespace(text.charCodeAt(i - 1))) {
                i--;
            }
            return i < 0 ? 0 : i;
        }

        private static function _isWhitespace(code:int):Boolean {
            return code == 0x20 || code == 0x09 || code == 0x0A || code == 0x0D || code == 0x2028 || code == 0x2029;
        }

        [API("670")]
        public function releaseLineCreationData():void {
            stub_method("flash.text.engine.TextBlock", "releaseLineCreationData");
        }

        public function dump():String {
            stub_method("flash.text.engine.TextBlock", "dump");
            return "<TextBlock>";
        }
    }
}
