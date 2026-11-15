package flash.display {

    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.events.NativeWindowBoundsEvent;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.desktop.NativeApplication;

    [API("661")]
    public class NativeWindow extends EventDispatcher {
        public const systemMaxSize:Point = new Point(2880, 2880);
        public const systemMinSize:Point = new Point(1, 1);
        public var minSize:Point = systemMinSize;
        public var maxSize:Point = systemMaxSize;
        public var title:String;
        public var alwaysInFront:Boolean = true;
        public var visible:Boolean = true;

        private var _bounds:Rectangle;
        private var _maximizable:Boolean;
        private var _minimizable:Boolean;
        private var _resizable:Boolean;
        private var _systemChrome:String;
        private var _transparent:Boolean;
        private var _type:String;
        private var _closed:Boolean = false;
        private var _stage:Stage;

        // TODO: FP does not have the `_stage` parameter, we should be constructing
        // the NativeWindow as a native object
        public function NativeWindow(initOptions:NativeWindowInitOptions, _stage:Stage = null) {
            NativeApplication.nativeApplication.openedWindows.push(this);
            if (_stage) {
                this._stage = _stage;
                _stage.addEventListener(Event.RESIZE, function(e:Event):void {
                    dispatchEvent(new NativeWindowBoundsEvent(NativeWindowBoundsEvent.RESIZE, false, false, _bounds, _bounds = new Rectangle(x, y, width, height)));
                });
            }

            _maximizable = initOptions.maximizable;
            _minimizable = initOptions.minimizable;
            _resizable = initOptions.resizable;
            _systemChrome = initOptions.systemChrome;
            _transparent = initOptions.transparent;
            _type = initOptions.type;
        }

        public function get width():Number {
            return _stage.stageWidth;
        }

        public function set width(value:Number):void {
            _stage.stageWidth = value;
        }

        public function get height():Number {
            return _stage.stageHeight;
        }

        public function set height(value:Number):void {
            _stage.stageHeight = value;
        }

        public function get x():Number {
            return _stage.x;
        }

        public function set x(value:Number):void {
        }

        public function get y():Number {
            return _stage.y;
        }

        public function set y(value:Number):void {
        }

        public function get bounds():Rectangle {
            return _bounds;
        }

        public function set bounds(value:Rectangle):void {
            _bounds = value;
        }

        public function get maximizable():Boolean {
            return _maximizable;
        }

        public function get minimizable():Boolean {
            return _minimizable;
        }

        public function get resizable():Boolean {
            return _resizable;
        }

        public function get systemChrome():String {
            return _systemChrome;
        }

        public function get transparent():Boolean {
            return _transparent;
        }

        public function get type():String {
            return _type;
        }

        public function get stage():Stage {
            return _stage;
        }

        public function activate():void {
            dispatchEvent(new Event(Event.ACTIVATE));
        }

        public function close():void {
            if (dispatchEvent(new Event(Event.CLOSING, false, true))) {
                _closed = true;
                dispatchEvent(new Event(Event.CLOSE));
                dispatchEvent(new Event(Event.DEACTIVATE));
            }
        }

        public function globalToScreen(globalPoint:Point):Point {
            return null;
        }

        [API("671")]
        public function listOwnedWindows():Vector.<NativeWindow> {
            return new Vector.<NativeWindow>();
        }

        public function maximize():void {
        }

        public function minimize():void {
        }

        public function notifyUser(type:String):void {
        }

        public function orderInBackOf(window:NativeWindow):Boolean {
            return false;
        }

        public function orderInFrontOf(window:NativeWindow):Boolean {
            return false;
        }

        public function orderToBack():Boolean {
            return false;
        }

        public function orderToFront():Boolean {
            return false;
        }

        public function restore():void {
        }

        public function startMove():Boolean {
            return false;
        }

        public function startResize(edgeOrCorner:String = "BR"):Boolean {
            return false;
        }

        public function get active():Boolean {
            return true;
        }

        public function get closed():Boolean {
            return this._closed;
        }

        public function get displayState():String {
            return "normal";
        }

        [API("668")]
        public function get isSupported():Boolean {
            return false;
        }

        [API("671")]
        public function get owner():NativeWindow {
            return this;
        }

        [API("675")]
        public function get renderMode():String {
            return "auto";
        }

        public function get supportsMenu():Boolean {
            return false;
        }

        public function get supportsNotification():Boolean {
            return false;
        }

        public function get supportsTransparency():Boolean {
            return false;
        }
    }
}
