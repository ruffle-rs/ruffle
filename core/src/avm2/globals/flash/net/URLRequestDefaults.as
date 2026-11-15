package flash.net {
    [API("661")]
    public class URLRequestDefaults {
        public static var _authenticate:Boolean = true;
        public static var _cacheResponse:Boolean = true;
        public static var _followRedirects:Boolean = true;
        public static var _idleTimeout:Number = 0;
        public static var _manageCookies:Boolean = true;
        public static var _useCache:Boolean = true;
        public static var _userAgent:String;

        public static function setLoginCredentialsForHost(hostname:String, user:String, password:String):* {
            return null;
        }

        public static function get authenticate():Boolean {
            return _authenticate;
        }

        public static function set authenticate(value:Boolean):void {
            _authenticate = value;
        }

        public static function get cacheResponse():Boolean {
            return _cacheResponse;
        }

        public static function set cacheResponse(value:Boolean):void {
            _cacheResponse = value;
        }

        public static function get followRedirects():Boolean {
            return _followRedirects;
        }

        public static function set followRedirects(value:Boolean):void {
            _followRedirects = value;
        }

        public static function get idleTimeout():Number {
            return _idleTimeout;
        }

        public static function set idleTimeout(value:Number):void {
            _idleTimeout = value;
        }

        public static function get manageCookies():Boolean {
            return _manageCookies;
        }

        public static function set manageCookies(value:Boolean):void {
            _manageCookies = value;
        }

        public static function get useCache():Boolean {
            return _useCache;
        }

        public static function set useCache(value:Boolean):void {
            _useCache = value;
        }

        public static function get userAgent():String {
            return _userAgent;
        }

        public static function set userAgent(value:String):void {
            _userAgent = value;
        }
    }
}
