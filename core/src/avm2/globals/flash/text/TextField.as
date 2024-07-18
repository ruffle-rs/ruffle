package flash.text {
    import flash.display.InteractiveObject;
    import flash.display.DisplayObject;
    import flash.geom.Rectangle;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class TextField extends InteractiveObject {
        internal var _styleSheet:StyleSheet;
        internal var _useRichTextClipboard:Boolean;

        public native function get alwaysShowSelection():Boolean;
        public native function set alwaysShowSelection(value:Boolean):void;

        public native function get autoSize():String;
        public native function set autoSize(value:String):void;

        public native function get background():Boolean;
        public native function set background(value:Boolean):void;

        public native function get backgroundColor():uint;
        public native function set backgroundColor(value:uint):void;

        public native function get border():Boolean;
        public native function set border(value:Boolean):void;

        public native function get borderColor():uint;
        public native function set borderColor(value:uint):void;

        public native function get bottomScrollV():int;

        public native function get condenseWhite():Boolean
        public native function set condenseWhite(value:Boolean):void

        public native function get defaultTextFormat():TextFormat;
        public native function set defaultTextFormat(value:TextFormat):void;

        public native function get displayAsPassword():Boolean;
        public native function set displayAsPassword(value:Boolean):void;

        public native function get embedFonts():Boolean;
        public native function set embedFonts(value:Boolean):void;

        public native function get htmlText():String;
        public native function set htmlText(value:String):void;

        public native function get length():int;

        public native function get maxScrollH():int;

        public native function get maxScrollV():int;

        public native function get maxChars():int;
        public native function set maxChars(value:int):void;

        public native function get mouseWheelEnabled():Boolean
        public native function set mouseWheelEnabled(value:Boolean):void

        public native function get multiline():Boolean;
        public native function set multiline(value:Boolean):void;

        public native function get restrict():String;
        public native function set restrict(value:String):void;

        public native function get scrollH():int;
        public native function set scrollH(value:int):void;

        public native function get scrollV():int;
        public native function set scrollV(value:int):void;

        public native function get selectable():Boolean;
        public native function set selectable(value:Boolean):void;

        public function get styleSheet():StyleSheet {
            return this._styleSheet;
        }
        public function set styleSheet(value:StyleSheet):void {
            this._styleSheet = value;
            stub_setter("flash.text.TextField", "styleSheet");
        }

        public native function get text():String;
        public native function set text(value:String):void;

        public native function get textColor():uint;
        public native function set textColor(value:uint):void;

        public native function get textHeight():Number;

        public native function get textWidth():Number;

        public native function get type():String;
        public native function set type(value:String):void;

        public function get useRichTextClipboard():Boolean {
            return this._useRichTextClipboard;
        }

        public function set useRichTextClipboard(value:Boolean):void {
            this._useRichTextClipboard = value;
            stub_setter("flash.text.TextField", "useRichTextClipboard");
        }

        public native function get wordWrap():Boolean;
        public native function set wordWrap(value:Boolean):void;

        public native function get antiAliasType():String;
        public native function set antiAliasType(value:String):void;

        public native function get gridFitType():String;
        public native function set gridFitType(value:String):void;

        public native function get thickness():Number;
        public native function set thickness(value:Number):void;

        public native function get sharpness():Number;
        public native function set sharpness(value:Number):void;

        public native function get numLines():int;

        public native function get caretIndex(): int;

        public native function get selectionBeginIndex(): int;
        public native function get selectionEndIndex(): int;

        public native function appendText(text:String):void;
        public native function getLineMetrics(lineIndex:int):TextLineMetrics;
        public native function getTextFormat(beginIndex:int = -1, endIndex:int = -1):TextFormat;
        public native function setTextFormat(format:TextFormat, beginIndex:int = -1, endIndex:int = -1):void;
        public native function replaceSelectedText(value:String):void;
        public native function replaceText(beginIndex:int, endIndex:int, newText:String):void;
        public native function setSelection(beginIndex:int, endIndex:int):void;
        public native function getTextRuns():Array;

        public native function get selectedText():String;

        public function insertXMLText(beginIndex:int, endIndex:int, text:String, paste:Boolean = false):void {
            stub_method("flash.text.TextField", "insertXMLText");
        }

        public native function getCharIndexAtPoint(x:Number, y:Number):int;

        public native function getLineLength(lineIndex:int):int;

        public native function getLineText(lineIndex:int):String;

        public native function getLineOffset(lineIndex:int):int;

        public function getCharBoundaries(charIndex:int):Rectangle {
            stub_method("flash.text.TextField", "getCharBoundaries");
            return new Rectangle(0, 0, 1, 1);
        }

        public native function getFirstCharInParagraph(charIndex:int):int;

        public function getImageReference(id:String):DisplayObject {
            stub_method("flash.text.TextField", "getImageReference");
            return null;
        }

        public native function getLineIndexAtPoint(x:Number, y:Number):int;

        public native function getLineIndexOfChar(charIndex:int):int;

        public native function getParagraphLength(charIndex:int):int;

        public static function isFontCompatible(fontName:String, fontStyle:String):Boolean {
            stub_method("flash.text.TextField", "isFontCompatible");
            return true;
        }

        public function get textInteractionMode():String {
            stub_getter("flash.text.TextField", "textInteractionMode");
            return TextInteractionMode.NORMAL;
        }
    }
}
