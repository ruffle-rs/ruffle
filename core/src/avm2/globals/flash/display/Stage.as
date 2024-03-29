package flash.display {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import flash.accessibility.AccessibilityProperties;
    import flash.display.ColorCorrection;
    import flash.display.ColorCorrectionSupport;
    import flash.errors.IllegalOperationError;
    import flash.filters.BitmapFilter;
    import flash.geom.Rectangle;
    import flash.geom.Transform;
    import flash.ui.ContextMenu;

    [Ruffle(NativeInstanceInit)]
    public class Stage extends DisplayObjectContainer {
        private var _colorCorrection:String = ColorCorrection.DEFAULT;

        public function Stage() {
            throw new Error("You cannot construct new instances of the Stage.")
        }

        override public function set accessibilityProperties(value:AccessibilityProperties):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set alpha(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set blendMode(value:String):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set cacheAsBitmap(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set contextMenu(value:ContextMenu):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set filters(value:Array):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set focusRect(value:Object):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        public function set loaderInfo(value:LoaderInfo):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set mask(value:DisplayObject):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set mouseEnabled(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function get name():String {
            return null;
        }
        
        override public function set name(value:String):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set opaqueBackground(value:Object):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set rotation(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set scale9Grid(value:Rectangle):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set scaleX(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set scaleY(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set scrollRect(value:Rectangle):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set tabEnabled(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set tabIndex(value:int):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set transform(value: Transform):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set visible(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set x(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }
 
        override public function set y(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        // End of overrides

        public native function get align():String;
        public native function set align(value:String):void;

        public native function get browserZoomFactor():Number;

        public native function get color():uint;
        public native function set color(value:uint):void;

        public native function get contentsScaleFactor():Number;

        public native function get displayState():String;
        public native function set displayState(value:String):void;

        public native function get focus():InteractiveObject;
        public native function set focus(value:InteractiveObject):void;

        public native function get frameRate():Number;
        public native function set frameRate(value:Number):void;

        public native function get fullScreenHeight():uint;

        public native function get fullScreenSourceRect():Rectangle;
        public native function set fullScreenSourceRect(value:Rectangle):void;

        public native function get fullScreenWidth():uint;

        public native function get scaleMode():String;
        public native function set scaleMode(value:String):void;

        public native function get showDefaultContextMenu():Boolean;
        public native function set showDefaultContextMenu(value:Boolean):void;

        public native function get stageWidth():int;
        public native function set stageWidth(value:int):void;

        public native function get stageHeight():int;
        public native function set stageHeight(value:int):void;

        public native function get stageFocusRect():Boolean;
        public native function set stageFocusRect(value:Boolean):void;

        public native function get allowsFullScreen():Boolean;

        public native function get allowsFullScreenInteractive():Boolean;

        public native function get quality():String;
        public native function set quality(value:String):void;

        public native function get stage3Ds():Vector.<Stage3D>;

        public native function invalidate():void;

        public function get colorCorrection():String {
            return this._colorCorrection;
        }
        public function set colorCorrection(value:String):void {
            stub_setter("flash.display.Stage", "colorCorrection");
            if (value == null) throw new TypeError("Error #2007: Parameter colorCorrection must be non-null.", 2007);
            this._colorCorrection = value;
        }

        public function get colorCorrectionSupport():String {
            stub_getter("flash.display.Stage", "colorCorrectionSupport");
            return ColorCorrectionSupport.UNSUPPORTED;
        }
    }
}
