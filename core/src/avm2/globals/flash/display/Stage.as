package flash.display {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import flash.accessibility.AccessibilityProperties;
    import flash.errors.IllegalOperationError;
    import flash.events.Event;
    import flash.geom.Rectangle;
    import flash.geom.Transform;
    import flash.text.TextSnapshot;
    import flash.ui.ContextMenu;

    [Ruffle(NativeInstanceInit)]
    public class Stage extends DisplayObjectContainer {
        private var _colorCorrection:String = ColorCorrection.DEFAULT;
        private var _mouseLock:Boolean = false;

        public function Stage() {
            throw new Error("You cannot construct new instances of the Stage.")
        }

        override public function set accessibilityProperties(value:AccessibilityProperties):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function addChild(child:DisplayObject):DisplayObject {
            return super.addChild(child);
        }

        override public function addChildAt(child:DisplayObject, index:int):DisplayObject {
            return super.addChildAt(child, index);
        }

        override public function addEventListener(type:String, listener:Function, useCapture:Boolean = false, priority:int = 0, useWeakReference:Boolean = false):void {
            super.addEventListener(type, listener, useCapture, priority, useWeakReference);
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

        override public function dispatchEvent(event:Event):Boolean {
            return super.dispatchEvent(event);
        }

        override public function set filters(value:Array):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set focusRect(value:Object):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function hasEventListener(type:String):Boolean {
            return super.hasEventListener(type);
        }

        override public function get height():Number {
            return super.height;
        }

        override public function set height(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set mask(value:DisplayObject):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function get mouseChildren():Boolean {
            return super.mouseChildren;
        }

        override public function set mouseChildren(value:Boolean):void {
            super.mouseChildren = value;
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

        override public function get numChildren():int {
            return super.numChildren;
        }

        override public function set opaqueBackground(value:Object):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function removeChildAt(index:int):DisplayObject {
            return super.removeChildAt(index);
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

        override public function setChildIndex(child:DisplayObject, index:int):void {
            super.setChildIndex(child, index);
        }

        override public function swapChildrenAt(index1:int, index2:int):void {
            super.swapChildrenAt(index1, index2);
        }

        override public function get tabChildren():Boolean {
            // stage.tabChildren is always true,
            // even if its setter was called with false
            return true;
        }

        override public native function set tabChildren(value:Boolean):void;

        override public function set tabEnabled(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set tabIndex(value:int):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function get textSnapshot():TextSnapshot {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set transform(value: Transform):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function set visible(value:Boolean):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function get width():Number {
            return super.width;
        }

        override public function set width(value:Number):void {
            throw new IllegalOperationError("Error #2071: The Stage class does not implement this property or method.", 2071);
        }

        override public function willTrigger(type:String):Boolean {
            return super.willTrigger(type);
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

        [API("670")]
        public function get softKeyboardRect() : Rectangle {
            stub_getter("flash.display.Stage", "softKeyboardRect");
            // This is technically a valid implementation most of the time,
            // as 0x0 Rect is the expected value with no soft keyboard.
            return new Rectangle(0, 0, 0, 0);
        }

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

        public function get mouseLock():Boolean {
            stub_getter("flash.display.Stage", "mouseLock");
            return this._mouseLock;
        }

        public function set mouseLock(value:Boolean):void {
            stub_setter("flash.display.Stage", "mouseLock");
            this._mouseLock = value;
        }

        [API("668")]
        public static function get supportsOrientationChange():Boolean {
            stub_getter("flash.display.Stage", "supportsOrientationChange");
            return false;
        }

        [API("671")]
        public function get supportedOrientations():Vector.<String> {
            stub_getter("flash.display.Stage", "supportedOrientations");
            return new Vector.<String>();
        }

        [API("668")]
        public function get autoOrients():Boolean {
            stub_getter("flash.display.Stage", "autoOrients");
            return false;
        }

        [API("668")]
        public function set autoOrients(value:Boolean):void {
            stub_setter("flash.display.Stage", "autoOrients");
        }

        [API("668")]
        public function get orientation():String {
            stub_getter("flash.display.Stage", "orientation");
            return StageOrientation.UNKNOWN;
        }

        [API("668")]
        public function get deviceOrientation():String {
            stub_getter("flash.display.Stage", "deviceOrientation");
            return StageOrientation.UNKNOWN;
        }

        [API("668")]
        public function setOrientation(newOrientation:String):void {
            stub_method("flash.display.Stage", "setOrientation");
        }

        [API("668")]
        public function setAspectRatio(newAspectRatio:String):void {
            stub_method("flash.display.Stage", "setAspectRatio");
        }
    }
}
