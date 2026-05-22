// Definitions for the flash.text.engine.TextLine class.
//
// The metric and atom accessors are native functions backed by the
// FteTextLine DisplayObject (core/src/display_object/fte_text_line.rs).
package flash.text.engine {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.errors.IllegalOperationError;
    import flash.events.EventDispatcher;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.ui.ContextMenu;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [Ruffle(Abstract)]
    [API("662")]
    public final class TextLine extends DisplayObjectContainer {
        public static const MAX_LINE_WIDTH:int = 1000000;

        // Adobe declares `public var userData:*;`. We back it with a
        // [NativeAccessible] slot so Rust code can read/write it.
        [Ruffle(NativeAccessible)]
        private var _userData:* = null;

        // Internal slots backing the AS3-side state; not part of the
        // public API but accessed by text_block.rs.
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

        [Ruffle(NativeAccessible)]
        internal var _validity:String = "valid";

        public function get userData():* {
            return this._userData;
        }

        public function set userData(value:*):void {
            this._userData = value;
        }

        public function get specifiedWidth():Number {
            return this._specifiedWidth;
        }

        public function get textBlock():TextBlock {
            return this._textBlock;
        }

        public function get rawTextLength():int {
            return this._rawTextLength;
        }

        public function get textBlockBeginIndex():int {
            return this._textBlockBeginIndex;
        }

        public function get nextLine():TextLine {
            return this._nextLine;
        }

        public function get previousLine():TextLine {
            return this._previousLine;
        }

        public function get validity():String {
            return this._validity;
        }

        public function set validity(value:String):void {
            this._validity = value;
        }

        // Metric and atom accessors, backed natively by FteTextLine.
        public native function get ascent():Number;
        public native function get descent():Number;
        public native function get textWidth():Number;
        public native function get textHeight():Number;
        public native function get atomCount():int;
        public native function getBaselinePosition(baseline:String):Number;
        public native function getAtomBounds(index:int):Rectangle;
        public native function getAtomCenter(index:int):Number;
        public native function getAtomBidiLevel(index:int):int;
        public native function getAtomIndexAtCharIndex(charIndex:int):int;
        public native function getAtomTextBlockBeginIndex(index:int):int;
        public native function getAtomTextBlockEndIndex(index:int):int;

        // Inline graphics are not laid out, so a line never contains a
        // GraphicElement.
        public function get hasGraphicElement():Boolean {
            stub_getter("flash.text.engine.TextLine", "hasGraphicElement");
            return false;
        }

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

        // The line is never justified, so its unjustified width is just the
        // measured width.
        public function get unjustifiedTextWidth():Number {
            return this.textWidth;
        }

        // The total metrics add the extents of inline graphics onto the text
        // metrics. With no inline graphics they equal the text ascent and
        // descent.
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
            return this.ascent + this.descent;
        }

        public function get mirrorRegions():Vector.<TextLineMirrorRegion> {
            stub_getter("flash.text.engine.TextLine", "mirrorRegions");
            return new Vector.<TextLineMirrorRegion>();
        }

        public function getMirrorRegion(mirror:EventDispatcher):TextLineMirrorRegion {
            var mr:Vector.<TextLineMirrorRegion> = this.mirrorRegions;
            for (var i:int = 0; i < mr.length; i++) {
                if (mr[i].mirror == mirror) {
                    return mr[i];
                }
            }
            return null;
        }

        // Hit testing: convert the global point into the line's own space and
        // return the first atom whose bounds contain it, or -1 for none.
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

        // A horizontal line never rotates its atoms. Verified against Flash
        // Player: getAtomTextRotation is rotate0 for every atom of one.
        public function getAtomTextRotation(index:int):String {
            return TextRotation.ROTATE_0;
        }

        public native function getAtomWordBoundaryOnLeft(index:int):Boolean;

        public function getAtomGraphic(index:int):DisplayObject {
            stub_method("flash.text.engine.TextLine", "getAtomGraphic");
            return null;
        }

        // Deprecated since FP 10.1; was always a cache hint.
        public function flushAtomData():void {}

        public function dump():String {
            return "<TextLine atomCount=" + this.atomCount + " textWidth=" + this.textWidth + ">";
        }

        // TextLine forbids these inherited setters.
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
    }
}
