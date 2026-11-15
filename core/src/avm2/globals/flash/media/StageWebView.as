package flash.media {
    import flash.display.BitmapData;
    import flash.display.Stage;
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    [API("669")]
    public final class StageWebView extends EventDispatcher {
        private var _stage:Stage = null;
        private var _viewPort:Rectangle = null;
        private var _location:String = "";
        private var _title:String = "";
        private var _mediaPlaybackRequiresUserAction:Boolean = true;

        // NOTE: Before AIR 51, the constructor was
        // `StageWebView(useNative:Boolean = false, mediaPlaybackRequiresUserAction:Boolean = true)`
        public function StageWebView(...configuration) {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function get stage():Stage {
            return this._stage;
        }
        public function set stage(value:Stage):void {
            this._stage = value;
        }

        public function get viewPort():Rectangle {
            return this._viewPort;
        }
        public function set viewPort(value:Rectangle):void {
            this._viewPort = value;
        }

        public function get location():String {
            return this._location;
        }

        public function get title():String {
            return this._title;
        }

        public function get isHistoryBackEnabled():Boolean {
            return false;
        }

        public function get isHistoryForwardEnabled():Boolean {
            return false;
        }

        public function get mediaPlaybackRequiresUserAction():Boolean {
            return this._mediaPlaybackRequiresUserAction;
        }
        public function set mediaPlaybackRequiresUserAction(value:Boolean):void {
            this._mediaPlaybackRequiresUserAction = value;
        }

        public function assignFocus(direction:String = "none"):void {
        }

        public function dispose():void {
        }

        public function drawViewPortToBitmapData(bitmap:BitmapData):void {
            if (bitmap == null) {
                Error.throwError(TypeError, 2007, "bitmap");
            }
        }

        public function historyBack():void {
        }

        public function historyForward():void {
        }

        public function loadString(text:String, mimeType:String = "text/html"):void {
            this._location = "about:blank";
        }

        public function loadURL(url:String):void {
            this._location = url;
        }

        public function reload():void {
        }

        public function stop():void {
        }
    }
}
