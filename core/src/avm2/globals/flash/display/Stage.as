package flash.display {
    import flash.accessibility.AccessibilityImplementation;
    import flash.accessibility.AccessibilityProperties;
    import flash.errors.IllegalOperationError;
    import flash.events.Event;
    import flash.geom.Rectangle;
    import flash.geom.Transform;
    import flash.media.StageVideo;
    import flash.text.TextSnapshot;
    import flash.ui.ContextMenu;

    [Ruffle(Abstract)]
    public class Stage extends DisplayObjectContainer {
        private var _colorCorrection:String = ColorCorrection.DEFAULT;
        private var _mouseLock:Boolean = false;
        private var _autoOrients:Boolean = false;
        private var _nativeWindow:NativeWindow;
        private var _fullScreenSourceRect:Rectangle;

        override public function set accessibilityImplementation(value:AccessibilityImplementation):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set accessibilityProperties(value:AccessibilityProperties):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function addChild(child:DisplayObject):DisplayObject {
            return super.addChild(child);
        }

        override public function addChildAt(child:DisplayObject, index:int):DisplayObject {
            return super.addChildAt(child, index);
        }

        override public function addEventListener(
            type:String,
            listener:Function,
            useCapture:Boolean = false,
            priority:int = 0,
            useWeakReference:Boolean = false
        ):void {
            super.addEventListener(type, listener, useCapture, priority, useWeakReference);
        }

        override public function set alpha(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set blendMode(value:String):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set cacheAsBitmap(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set contextMenu(value:ContextMenu):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function dispatchEvent(event:Event):Boolean {
            return super.dispatchEvent(event);
        }

        override public function set filters(value:Array):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set focusRect(value:Object):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function hasEventListener(type:String):Boolean {
            return super.hasEventListener(type);
        }

        override public function get height():Number {
            return super.height;
        }

        override public function set height(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set mask(value:DisplayObject):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function get mouseChildren():Boolean {
            return super.mouseChildren;
        }

        override public function set mouseChildren(value:Boolean):void {
            super.mouseChildren = value;
        }

        override public function set mouseEnabled(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set name(value:String):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function get numChildren():int {
            return super.numChildren;
        }

        override public function set opaqueBackground(value:Object):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function removeChildAt(index:int):DisplayObject {
            return super.removeChildAt(index);
        }

        override public function set rotation(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        [API("662")]
        override public function set rotationX(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        [API("662")]
        override public function set rotationY(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        [API("662")]
        override public function set rotationZ(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set scale9Grid(value:Rectangle):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set scaleX(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set scaleY(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        [API("662")]
        override public function set scaleZ(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set scrollRect(value:Rectangle):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function setChildIndex(child:DisplayObject, index:int):void {
            super.setChildIndex(child, index);
        }

        override public function swapChildrenAt(index1:int, index2:int):void {
            super.swapChildrenAt(index1, index2);
        }

        override public function get tabChildren():Boolean {
            return true;
        }

        override public native function set tabChildren(value:Boolean):void;

        override public function set tabEnabled(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set tabIndex(value:int):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function get textSnapshot():TextSnapshot {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set transform(value:Transform):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set visible(value:Boolean):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function get width():Number {
            return super.width;
        }

        override public function set width(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function willTrigger(type:String):Boolean {
            return super.willTrigger(type);
        }

        override public function set x(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        override public function set y(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        [API("662")]
        override public function set z(value:Number):void {
            Error.throwError(IllegalOperationError, 2071);
        }

        // End of overrides

        public native function get align():String;
        public native function set align(value:String):void;

        [API("700")]
        public native function get browserZoomFactor():Number;

        [API("670")]
        public native function get color():uint;
        [API("670")]
        public native function set color(value:uint):void;

        [API("682")]
        public native function get contentsScaleFactor():Number;

        public native function get displayState():String;
        public native function set displayState(value:String):void;

        public native function get focus():InteractiveObject;
        public native function set focus(value:InteractiveObject):void;

        public function isFocusInaccessible():Boolean {
            return false;
        }

        public native function get frameRate():Number;
        public native function set frameRate(value:Number):void;

        public native function get fullScreenHeight():uint;

        public function get fullScreenSourceRect():Rectangle {
            return this._fullScreenSourceRect;
        }

        public function set fullScreenSourceRect(rect:Rectangle):void {
            this._fullScreenSourceRect = rect;
        }

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

        [API("665")]
        public function get wmodeGPU():Boolean {
            return true;
        }

        [API("670")]
        public function get softKeyboardRect():Rectangle {
            return new Rectangle(0, 0, 0, 0);
        }

        [API("670")]
        public native function get allowsFullScreen():Boolean;

        [API("680")]
        public native function get allowsFullScreenInteractive():Boolean;

        public native function get quality():String;
        public native function set quality(value:String):void;

        [API("667")]
        public function get stageVideos():Vector.<StageVideo> {
            return new Vector.<StageVideo>();
        }
        
        [API("674")]
        public native function get stage3Ds():Vector.<Stage3D>;
        public native function invalidate():void;
        
        [API("662")]
        public function get colorCorrection():String {
        return this._colorCorrection;
        }

        [API("662")]
        public function set colorCorrection(value:String):void {
        if (value == null) {
            Error.throwError(TypeError, 2007, "colorCorrection");
        }
        if (
        value != ColorCorrection.DEFAULT &&
        value != ColorCorrection.ON &&
        value != ColorCorrection.OFF
        ) {
        Error.throwError(ArgumentError, 2008, "colorCorrection");
        }
        this._colorCorrection = value;
        }
        
        [API("662")]
        public function get colorCorrectionSupport():String {
        return ColorCorrectionSupport.UNSUPPORTED;
        }
        
        [API("678")]
        public function get mouseLock():Boolean {
        return this._mouseLock;
        }
        
        [API("678")]
        public function set mouseLock(value:Boolean):void {
        this._mouseLock = value;
        }
        
        [API("668")]
        public static function get supportsOrientationChange():Boolean {
        return false;
        }
        
        [API("671")]
        public function get supportedOrientations():Vector.<String> {
            return new Vector.<String>();
        }
        
        [API("668")]
        public function get autoOrients():Boolean {
        return this._autoOrients;
        }
        
        [API("668")]
        public function set autoOrients(value:Boolean):void {
        this._autoOrients = value;
        }

        [API("668")]
        public function get orientation():String {
        return StageOrientation.UNKNOWN;
        }
        
        [API("668")]
        public function get deviceOrientation():String {
        return StageOrientation.UNKNOWN;
        }
        
        [API("668")]
        public function setOrientation(newOrientation:String):void {
        }
        
        [API("668")]
        public function setAspectRatio(newAspectRatio:String):void {
        }
        
        [API("674")]
        public function get displayContextInfo():String {
        return null;
        }
        
        public function get constructor():* {
            return null;
        }
        
        public function set constructor(value:*):* {
        }
        
        [API("661")]
        public function get nativeWindow():NativeWindow {
        if (!this._nativeWindow) {
        this._nativeWindow = new NativeWindow(new NativeWindowInitOptions(), this);
        }
        return this._nativeWindow;
        }
    }
}
