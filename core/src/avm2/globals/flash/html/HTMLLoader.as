package flash.html {
    import flash.display.Sprite;
    import flash.net.URLRequest;

    [API("661")]
    public class HTMLLoader extends Sprite {
        private var _location:String = "";
        private var _paintsDefaultBackground:Boolean = true;
        private var _useCache:Boolean = true;
        private var _cacheResponse:Boolean = true;
        private var _userAgent:String = "";
        private var _manageCookies:Boolean = true;
        private var _authenticate:Boolean = true;
        private var _navigateInSystemBrowser:Boolean = false;
        private var _placeLoadStringContentInApplicationSandbox:Boolean = false;
        private var _scrollH:Number = 0;
        private var _scrollV:Number = 0;
        private var _textEncodingOverride:String = null;
        private var _textEncodingFallback:String = null;
        private var _runtimeApplicationDomain:Object = null;

        public function HTMLLoader() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public static function get pdfCapability():int {
            return 3201;
        }

        public function get location():String {
            return this._location;
        }

        public function get loaded():Boolean {
            return true;
        }

        public function get contentHeight():Number {
            return this.height;
        }

        public function get contentWidth():Number {
            return this.width;
        }

        public function get historyLength():uint {
            return 0;
        }

        public function get historyPosition():uint {
            return 0;
        }
        public function set historyPosition(value:uint):void {
        }

        public function get paintsDefaultBackground():Boolean {
            return this._paintsDefaultBackground;
        }
        public function set paintsDefaultBackground(value:Boolean):void {
            this._paintsDefaultBackground = value;
        }

        public function get scrollH():Number {
            return this._scrollH;
        }
        public function set scrollH(value:Number):void {
            this._scrollH = value;
        }

        public function get scrollV():Number {
            return this._scrollV;
        }
        public function set scrollV(value:Number):void {
            this._scrollV = value;
        }

        public function get userAgent():String {
            return this._userAgent;
        }
        public function set userAgent(value:String):void {
            this._userAgent = value;
        }

        public function get useCache():Boolean {
            return this._useCache;
        }
        public function set useCache(value:Boolean):void {
            this._useCache = value;
        }

        public function get cacheResponse():Boolean {
            return this._cacheResponse;
        }
        public function set cacheResponse(value:Boolean):void {
            this._cacheResponse = value;
        }

        public function get manageCookies():Boolean {
            return this._manageCookies;
        }
        public function set manageCookies(value:Boolean):void {
            this._manageCookies = value;
        }

        public function get authenticate():Boolean {
            return this._authenticate;
        }
        public function set authenticate(value:Boolean):void {
            this._authenticate = value;
        }

        public function get navigateInSystemBrowser():Boolean {
            return this._navigateInSystemBrowser;
        }
        public function set navigateInSystemBrowser(value:Boolean):void {
            this._navigateInSystemBrowser = value;
        }

        public function get placeLoadStringContentInApplicationSandbox():Boolean {
            return this._placeLoadStringContentInApplicationSandbox;
        }
        public function set placeLoadStringContentInApplicationSandbox(value:Boolean):void {
            this._placeLoadStringContentInApplicationSandbox = value;
        }

        public function get textEncodingOverride():String {
            return this._textEncodingOverride;
        }
        public function set textEncodingOverride(value:String):void {
            this._textEncodingOverride = value;
        }

        public function get textEncodingFallback():String {
            return this._textEncodingFallback;
        }
        public function set textEncodingFallback(value:String):void {
            this._textEncodingFallback = value;
        }

        public function get runtimeApplicationDomain():Object {
            return this._runtimeApplicationDomain;
        }
        public function set runtimeApplicationDomain(value:Object):void {
            this._runtimeApplicationDomain = value;
        }

        public function get window():Object {
            return null;
        }

        public function get hasFocusableContent():Boolean {
            return false;
        }

        public function cancelLoad():void {
        }

        public function historyBack():void {
        }

        public function historyForward():void {
        }

        public function historyGo(steps:int):void {
        }

        public function load(urlRequestToLoad:URLRequest):void {
            if (urlRequestToLoad != null) {
                this._location = urlRequestToLoad.url;
            }
        }

        public function loadString(htmlContent:String):void {
            this._location = "about:blank";
        }

        public function reload():void {
        }
    }
}
