package flash.display {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_constructor;

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
            stub_constructor("flash.display.NativeWindow");
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
            stub_getter("flash.display.NativeWindow", "width");
            return _stage.stageWidth;
        }

        public function set width(value:Number):void {
            stub_setter("flash.display.NativeWindow", "width");
            _stage.stageWidth = value;
        }

        public function get height():Number {
            stub_getter("flash.display.NativeWindow", "height");
            return _stage.stageHeight;
        }

        public function set height(value:Number):void {
            stub_setter("flash.display.NativeWindow", "height");
            _stage.stageHeight = value;
        }

        public function get x():Number {
            stub_getter("flash.display.NativeWindow", "x");
            return _stage.x;
        }

        public function set x(value:Number):void {
            stub_setter("flash.display.NativeWindow", "x");
        }

        public function get y():Number {
            stub_getter("flash.display.NativeWindow", "y");
            return _stage.y;
        }

        public function set y(value:Number):void {
            stub_setter("flash.display.NativeWindow", "y");
        }

        public function get bounds():Rectangle {
            stub_getter("flash.display.NativeWindow", "bounds");
            return _bounds;
        }

        public function set bounds(value:Rectangle):void {
            stub_setter("flash.display.NativeWindow", "bounds");
            _bounds = value;
        }

        public function get maximizable():Boolean {
            stub_getter("flash.display.NativeWindow", "maximizable");
            return _maximizable;
        }

        public function get minimizable():Boolean {
            stub_getter("flash.display.NativeWindow", "minimizable");
            return _minimizable;
        }

        public function get resizable():Boolean {
            stub_getter("flash.display.NativeWindow", "resizable");
            return _resizable;
        }

        public function get systemChrome():String {
            stub_getter("flash.display.NativeWindow", "systemChrome");
            return _systemChrome;
        }

        public function get transparent():Boolean {
            stub_getter("flash.display.NativeWindow", "transparent");
            return _transparent;
        }

        public function get type():String {
            stub_getter("flash.display.NativeWindow", "type");
            return _type;
        }

        public function get stage():Stage {
            return _stage;
        }

        public function activate():void {
            stub_method("flash.display.NativeWindow", "activate");
            dispatchEvent(new Event(Event.ACTIVATE));
        }

        public function close():void {
            stub_method("flash.display.NativeWindow", "close");
            if (dispatchEvent(new Event(Event.CLOSING, false, true))) {
                _closed = true;
                dispatchEvent(new Event(Event.CLOSE));
                dispatchEvent(new Event(Event.DEACTIVATE));
            }
        }

        public function globalToScreen(globalPoint:Point):Point {
            stub_method("flash.display.NativeWindow", "globalToScreen");
            return null;
        }

        [API("671")]
        public function listOwnedWindows():Vector.<NativeWindow> {
            stub_method("flash.display.NativeWindow", "listOwnedWindows");
            return new Vector.<NativeWindow>();
        }

        public function maximize():void {
            stub_method("flash.display.NativeWindow", "maximize");
        }

        public function minimize():void {
            stub_method("flash.display.NativeWindow", "minimize");
        }

        public function notifyUser(type:String):void {
            stub_method("flash.display.NativeWindow", "notifyUser");
        }

        public function orderInBackOf(window:NativeWindow):Boolean {
            stub_method("flash.display.NativeWindow", "orderInBackOf");
            return false;
        }

        public function orderInFrontOf(window:NativeWindow):Boolean {
            stub_method("flash.display.NativeWindow", "orderInFrontOf");
            return false;
        }

        public function orderToBack():Boolean {
            stub_method("flash.display.NativeWindow", "orderToBack");
            return false;
        }

        public function orderToFront():Boolean {
            stub_method("flash.display.NativeWindow", "orderToFront");
            return false;
        }

        public function restore():void {
            stub_method("flash.display.NativeWindow", "restore");
        }

        public function startMove():Boolean {
            stub_method("flash.display.NativeWindow", "startMove");
            return false;
        }

        public function startResize(edgeOrCorner:String = "BR"):Boolean {
            stub_method("flash.display.NativeWindow", "startResize");
            return false;
        }

        public function get active():Boolean {
            stub_getter("flash.display.NativeWindow", "active");
            return true;
        }

        public function get closed():Boolean {
            return this._closed;
        }

        public function get displayState():String {
            stub_getter("flash.display.NativeWindow", "displayState");
            return "normal";
        }

        [API("668")]
        public function get isSupported():Boolean {
            stub_getter("flash.display.NativeWindow", "isSupported");
            return false;
        }

        [API("671")]
        public function get owner():NativeWindow {
            stub_getter("flash.display.NativeWindow", "owner");
            return this;
        }

        [API("675")]
        public function get renderMode():String {
            stub_getter("flash.display.NativeWindow", "renderMode");
            return "auto";
        }

        public function get supportsMenu():Boolean {
            stub_getter("flash.display.NativeWindow", "supportsMenu");
            return false;
        }

        public function get supportsNotification():Boolean {
            stub_getter("flash.display.NativeWindow", "supportsNotification");
            return false;
        }

        public function get supportsTransparency():Boolean {
            stub_getter("flash.display.NativeWindow", "supportsTransparency");
            return false;
        }
    }
}
