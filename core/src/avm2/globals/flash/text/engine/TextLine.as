package flash.text.engine {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_method;

    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.errors.IllegalOperationError;
    import flash.geom.Point;
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
        private var _textBlockBeginIndex:int = 0;

        [Ruffle(NativeAccessible)]
        internal var _nextLine:TextLine = null;

        [Ruffle(NativeAccessible)]
        internal var _previousLine:TextLine = null;

        internal var _validity:String = "valid";

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

        public function get nextLine():TextLine {
            return this._nextLine;
        }

        public function get previousLine():TextLine {
            return this._previousLine;
        }

        public native function get ascent():Number;

        [API("670")]
        public function get totalAscent():Number {
            return this.ascent;
        }

        public native function get descent():Number;

        [API("670")]
        public function get totalDescent():Number {
            return this.descent;
        }

        public function get unjustifiedTextWidth():Number {
            return this.textWidth;
        }

        public native function get textWidth():Number;
        public native function get textHeight():Number;

        [API("670")]
        public function get totalHeight():Number {
            return this.ascent + this.descent;
        }

        public function get validity():String {
            stub_getter("flash.text.engine.TextLine", "validity");
            return this._validity;
        }

        public function set validity(value:String):void {
            stub_setter("flash.text.engine.TextLine", "validity");
            this._validity = value;
        }

        public function get hasGraphicElement():Boolean {
            stub_getter("flash.text.engine.TextLine", "hasGraphicElement");
            return false;
        }

        public native function get atomCount():int;

        public native function getBaselinePosition(baseline:String):Number;

        public function get hasTabs():Boolean {
            var block:TextBlock = this._textBlock;
            if (block == null || block.content == null) {
                return false;
            }
            var text:String = block.content.text;
            if (text == null) {
                return false;
            }
            var end:int = this._textBlockBeginIndex + this._rawTextLength;
            if (end > text.length) {
                end = text.length;
            }
            for (var i:int = this._textBlockBeginIndex; i < end; i++) {
                if (text.charCodeAt(i) == 0x09) {
                    return true;
                }
            }
            return false;
        }

        public function getAtomIndexAtPoint(stageX:Number, stageY:Number):int {
            var p:Point = this.globalToLocal(new Point(stageX, stageY));
            var n:int = this.atomCount;
            for (var i:int = 0; i < n; i++) {
                if (this.getAtomBounds(i).containsPoint(p)) {
                    return i;
                }
            }
            return -1;
        }

        public native function getAtomIndexAtCharIndex(charIndex:int):int;

        public function getAtomBidiLevel(index:int):int {
            stub_method("flash.text.engine.TextLine", "getAtomBidiLevel");
            return 0;
        }

        public native function getAtomBounds(index:int):Rectangle;

        public native function getAtomCenter(index:int):Number;

        public function getAtomGraphic(index:int):DisplayObject {
            stub_method("flash.text.engine.TextLine", "getAtomGraphic");
            return null;
        }

        public native function getAtomTextBlockBeginIndex(index:int):int;

        public native function getAtomTextBlockEndIndex(index:int):int;

        public function getAtomTextRotation(index:int):String {
            stub_method("flash.text.engine.TextLine", "getAtomTextRotation");
            return TextRotation.ROTATE_0;
        }

        public native function getAtomWordBoundaryOnLeft(index:int):Boolean;

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
