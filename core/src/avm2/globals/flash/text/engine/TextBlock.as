package flash.text.engine {
    import __ruffle__.stub_getter;
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

        public function recreateTextLine(
            textLine:TextLine,
            previousLine:TextLine = null,
            width:Number = 1000000,
            lineOffset:Number = 0,
            fitSomething:Boolean = false
        ):TextLine {
            if (textLine == null) {
                throw new ArgumentError("Error #2004: One of the parameters is invalid.", 2004);
            }

            if (previousLine) {
                return null;
            }

            stub_method("flash.text.engine.TextBlock", "recreateTextLine");

            // FIXME: Properly recalculate new properties of new TextLine. Text layout
            // modules often depend on this returning the same textLine, so we can't
            // call `createTextLine` again.
            return textLine;
        }

        public native function get textLineCreationResult():String;

        public native function get firstInvalidLine():TextLine;

        public native function get firstLine():TextLine;

        public function get lastLine():TextLine {
            stub_getter("flash.text.engine.TextBlock", "lastLine");
            return this.firstLine;
        }

        public function releaseLines(start:TextLine, end:TextLine):void {
            if (start != end || end != this.firstLine) {
                stub_method("flash.text.engine.TextBlock", "releaseLines", "with start != end or multiple lines");
                return;
            }
            this.firstLine.validity = "invalid";
            this.firstLine._textBlock = null;
        }
    }
}
