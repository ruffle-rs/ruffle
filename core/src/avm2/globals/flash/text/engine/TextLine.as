package flash.text.engine {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.errors.IllegalOperationError;
    import flash.geom.Rectangle;
    import flash.ui.ContextMenu;

    // FIXME: None of the DisplayObjectContainer methods actually work on
    // the TextLine class in Ruffle, despite the methods working fine in FP-
    // however, it's unlikely that SWFs will actually attempt to add children
    // to a TextLine.
    [Ruffle(Abstract)]
    [API("662")]
    public final class TextLine extends DisplayObjectContainer {
        [Ruffle(NativeAccessible)]
        private var _specifiedWidth:Number = 0.0;

        [Ruffle(NativeAccessible)]
        internal var _textBlock:TextBlock = null;

        [Ruffle(NativeAccessible)]
        private var _rawTextLength:int = 0;

        [Ruffle(NativeAccessible)]
        internal var _textBlockBeginIndex:int = 0;

        [Ruffle(NativeAccessible)]
        internal var _nextLine:TextLine = null;

        [Ruffle(NativeAccessible)]
        internal var _previousLine:TextLine = null;

        public static const MAX_LINE_WIDTH:int = 1000000;

        public var userData;

        public function get rawTextLength():int {
            return this._rawTextLength;
        }

        public function get textBlockBeginIndex():int {
            return this._textBlockBeginIndex;
        }

        public function get specifiedWidth():Number {
            return this._specifiedWidth;
        }

        public function get textBlock():TextBlock {
            return this._textBlock;
        }

        public native function get ascent():Number;
        public native function get descent():Number;

        // TODO: totalAscent/totalDescent must also take GraphicElements into
        // account once GraphicElement is supported; for text-only lines they
        // equal ascent/descent.
        [API("670")]
        public function get totalAscent():Number {
            return this.ascent;
        }

        [API("670")]
        public function get totalDescent():Number {
            return this.descent;
        }

        [API("670")]
        public function get totalHeight():Number {
            return this.totalAscent + this.totalDescent;
        }

        public function get unjustifiedTextWidth():Number {
            // TODO: Return the pre-justification width once justification is supported.
            return this.textWidth;
        }

        public native function get textWidth():Number;
        public native function get textHeight():Number;

        public native function get validity():String;
        public native function set validity(value:String):void;

        public function get hasGraphicElement():Boolean {
            // TODO: Implement together with GraphicElement support.
            return false;
        }

        public function get atomCount():int {
            // TODO: This is an approximation- combining characters should
            // collapse into a single atom, and graphic elements count as one.
            return this._rawTextLength;
        }

        public function get nextLine():TextLine {
            return this._nextLine;
        }

        public function get previousLine():TextLine {
            return this._previousLine;
        }

        public function getBaselinePosition(baseline:String):Number {
            if (baseline == null) {
                throw new TypeError("Error #2007: Parameter baseline must be non-null.", 2007);
            }

            // Baseline positions are expressed in this line's coordinate
            // space, whose origin is the roman baseline (adjusted by the
            // block's baselineZero setting).
            var zero:String = TextBaseline.ROMAN;
            if (this._textBlock) {
                zero = this._textBlock.baselineZero;
            }

            return this.baselineOffset(baseline) - this.baselineOffset(zero);
        }

        private function baselineOffset(baseline:String):Number {
            // Offsets relative to the roman baseline, positive going down.
            switch (baseline) {
                case TextBaseline.ROMAN:
                    return 0.0;
                case TextBaseline.ASCENT:
                case TextBaseline.IDEOGRAPHIC_TOP:
                    return -this.ascent;
                case TextBaseline.DESCENT:
                case TextBaseline.IDEOGRAPHIC_BOTTOM:
                    return this.descent;
                case TextBaseline.IDEOGRAPHIC_CENTER:
                    return (this.descent - this.ascent) / 2;
                default:
                    throw new ArgumentError("Error #2008: Parameter baseline must be one of the accepted values.", 2008);
            }
        }

        public function get hasTabs():Boolean {
            // TODO: Implement together with TabStop support.
            return false;
        }

        public native function getAtomIndexAtPoint(stageX:Number, stageY:Number):int;

        public function getAtomIndexAtCharIndex(charIndex:int):int {
            // charIndex is an index into the parent TextBlock; map it to a
            // zero-based atom index relative to this line, or -1 if it does
            // not belong to this line.
            // TODO: This is an approximation- combining characters should
            // collapse into a single atom, see atomCount.
            var relative:int = charIndex - this._textBlockBeginIndex;
            if (relative < 0 || relative >= this._rawTextLength) {
                return -1;
            }
            return relative;
        }

        public function getAtomBidiLevel(index:int):int {
            this.checkAtomIndex(index);
            // TODO: Implement together with bidi support.
            return 0;
        }

        public native function getAtomBounds(index:int):Rectangle;

        public function getAtomCenter(index:int):Number {
            var bounds:Rectangle = this.getAtomBounds(index);
            return bounds.x + bounds.width / 2;
        }

        public function getAtomGraphic(index:int):DisplayObject {
            this.checkAtomIndex(index);
            // TODO: Implement together with GraphicElement support.
            return null;
        }

        public function getAtomTextBlockBeginIndex(index:int):int {
            this.checkAtomIndex(index);
            return this._textBlockBeginIndex + index;
        }

        public function getAtomTextBlockEndIndex(index:int):int {
            this.checkAtomIndex(index);
            return this._textBlockBeginIndex + index + 1;
        }

        public function getAtomTextRotation(index:int):String {
            this.checkAtomIndex(index);
            // TODO: Implement together with textRotation support.
            return TextRotation.ROTATE_0;
        }

        public native function getAtomWordBoundaryOnLeft(index:int):Boolean;

        internal function checkAtomIndex(index:int):void {
            if (index < 0 || index >= this.atomCount) {
                throw new RangeError("Error #2006: The supplied index is out of bounds.", 2006);
            }
        }

        // This function does nothing in Flash Player 32
        public function flushAtomData():void { }

        // Overrides

        override public function set contextMenu(cm:ContextMenu):void {
            throw new IllegalOperationError("Error #2181: The TextLine class does not implement this property or method.", 2181);
        }

        override public function set focusRect(value:Object):void {
            throw new IllegalOperationError("Error #2181: The TextLine class does not implement this property or method.", 2181);
        }

        override public function set tabChildren(value:Boolean):void {
            throw new IllegalOperationError("Error #2181: The TextLine class does not implement this property or method.", 2181);
        }

        override public function set tabEnabled(value:Boolean):void {
            throw new IllegalOperationError("Error #2181: The TextLine class does not implement this property or method.", 2181);
        }

        override public function set tabIndex(index:int):void {
            throw new IllegalOperationError("Error #2181: The TextLine class does not implement this property or method.", 2181);
        }

        // End of overrides
    }
}
