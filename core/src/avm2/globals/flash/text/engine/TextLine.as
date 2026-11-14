package flash.text.engine {
    import flash.display.DisplayObject;
    import flash.display.DisplayObjectContainer;
    import flash.errors.IllegalOperationError;
    import flash.geom.Rectangle;
    import flash.ui.ContextMenu;

    [Ruffle(Abstract)]
    [API("662")]
    public final class TextLine extends DisplayObjectContainer {
        public static const MAX_LINE_WIDTH:int = 1000000;

        public var userData;

        public function get mirrorRegions():Vector.<TextLineMirrorRegion> {
            return null;
        }

        public native function get rawTextLength():int;

        public native function get textBlockBeginIndex():int;

        public native function get specifiedWidth():Number;

        public native function get textBlock():TextBlock;

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
            return this.specifiedWidth;
        }

        public native function get textWidth():Number;
        public native function get textHeight():Number;

        public native function get validity():String;
        public native function set validity(value:String):void;

        public function get hasGraphicElement():Boolean {
            return false;
        }

        public function get atomCount():int {
            return this.rawTextLength;
        }

        public native function get previousLine():TextLine;
        public native function get nextLine():TextLine;

        public function getBaselinePosition(baseline:String):Number {
            return 0.0;
        }

        public native function get hasTabs():Boolean;

        public function getAtomIndexAtPoint(stageX:Number, stageY:Number):int {
            return -1;
        }

        public function getAtomIndexAtCharIndex(charIndex:int):int {
            var index:int = charIndex - this.textBlockBeginIndex;
            if (index < 0 || index >= this.rawTextLength) {
                return -1;
            }
            return index;
        }

        private function checkAtomIndex(index:int):void {
            if (index < 0 || index >= this.atomCount) {
                Error.throwError(RangeError, 2006);
            }
        }

        public function getAtomBidiLevel(index:int):int {
            this.checkAtomIndex(index);
            return 0;
        }

        public function getAtomBounds(index:int):Rectangle {
            this.checkAtomIndex(index);
            return new Rectangle(0, 0, 0, 0);
        }

        public function getAtomCenter(index:int):Number {
            this.checkAtomIndex(index);
            return 0.0;
        }

        public function getAtomGraphic(index:int):DisplayObject {
            this.checkAtomIndex(index);
            return null;
        }

        public function getAtomTextBlockBeginIndex(index:int):int {
            this.checkAtomIndex(index);
            return this.textBlockBeginIndex + index;
        }

        public function getAtomTextBlockEndIndex(index:int):int {
            this.checkAtomIndex(index);
            return this.textBlockBeginIndex + index + 1;
        }

        public function getAtomTextRotation(index:int):String {
            this.checkAtomIndex(index);
            return TextRotation.ROTATE_0;
        }

        public function getAtomWordBoundaryOnLeft(index:int):Boolean {
            this.checkAtomIndex(index);
            return false;
        }

        public function flushAtomData():void { }

        override public function set contextMenu(cm:ContextMenu):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set focusRect(value:Object):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabChildren(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabEnabled(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2181);
        }

        override public function set tabIndex(index:int):void {
            Error.throwError(IllegalOperationError, 2181);
        }
    }
}
