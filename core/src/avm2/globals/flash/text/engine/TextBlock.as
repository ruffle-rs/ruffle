package flash.text.engine {
    import __ruffle__.stub_method;

    [API("662")]
    [Ruffle(InstanceAllocator)]
    public final class TextBlock {
        public var userData;

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
                this.textJustifier = new SpaceJustifier();
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

        public native function get applyNonLinearFontScaling():Boolean;
        public native function set applyNonLinearFontScaling(value:Boolean):void;

        public native function get baselineFontDescription():FontDescription;
        public native function set baselineFontDescription(value:FontDescription):void;

        public native function get baselineFontSize():Number;
        public native function set baselineFontSize(value:Number):void;

        public native function get baselineZero():String;
        public native function set baselineZero(value:String):void;

        public native function get bidiLevel():int;
        public native function set bidiLevel(value:int):void;

        public native function get lineRotation():String;
        public native function set lineRotation(value:String):void;

        public native function get tabStops():Vector.<TabStop>;
        public native function set tabStops(value:Vector.<TabStop>):void;

        public native function get textJustifier():TextJustifier;
        public function set textJustifier(value:TextJustifier):void {
            this.setTextJustifier(value);
        }

        private native function setTextJustifier(value:TextJustifier):void;

        public native function get content():ContentElement;
        public native function set content(value:ContentElement):void;

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

        public native function get textLineCreationResult():String;

        public native function get firstInvalidLine():TextLine;

        public native function get firstLine():TextLine;

        public native function get lastLine():TextLine;

        public native function releaseLines(firstLine:TextLine, lastLine:TextLine):void;

        public function findNextAtomBoundary(afterCharIndex:int):int {
            // TODO: This is an approximation- combining characters should be
            // skipped over, as they belong to the previous atom.
            var text:String = this.contentText();
            if (afterCharIndex < 0 || afterCharIndex >= text.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            return afterCharIndex + 1;
        }

        public function findPreviousAtomBoundary(beforeCharIndex:int):int {
            // TODO: This is an approximation, see findNextAtomBoundary.
            var text:String = this.contentText();
            if (beforeCharIndex <= 0 || beforeCharIndex > text.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            return beforeCharIndex - 1;
        }

        public function findNextWordBoundary(afterCharIndex:int):int {
            var text:String = this.contentText();
            if (afterCharIndex < 0 || afterCharIndex >= text.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            var wasSpace:Boolean = TextBlock.isWordSeparator(text.charCodeAt(afterCharIndex));
            for (var i:int = afterCharIndex + 1; i < text.length; i++) {
                var isSpace:Boolean = TextBlock.isWordSeparator(text.charCodeAt(i));
                if (isSpace != wasSpace) {
                    return i;
                }
            }
            return text.length;
        }

        public function findPreviousWordBoundary(beforeCharIndex:int):int {
            var text:String = this.contentText();
            if (beforeCharIndex <= 0 || beforeCharIndex > text.length) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
            var wasSpace:Boolean = TextBlock.isWordSeparator(text.charCodeAt(beforeCharIndex - 1));
            for (var i:int = beforeCharIndex - 1; i > 0; i--) {
                var isSpace:Boolean = TextBlock.isWordSeparator(text.charCodeAt(i - 1));
                if (isSpace != wasSpace) {
                    return i;
                }
            }
            return 0;
        }

        public function dump():String {
            stub_method("flash.text.engine.TextBlock", "dump");
            return "";
        }

        private static function isWordSeparator(charCode:uint):Boolean {
            return charCode == 0x20 || charCode == 0x09 || charCode == 0x0A
                || charCode == 0x0D || charCode == 0xA0 || charCode == 0x2028
                || charCode == 0x2029 || charCode == 0x3000;
        }

        private function contentText():String {
            var c:ContentElement = this.content;
            if (c == null || c.text == null) {
                return "";
            }
            return c.text;
        }
    }
}
