package flash.net {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public final class URLRequest {
        // NOTE - when implementing properties (e.g. `contentType`, `data`, etc.)
        // be sure to also check for them in `URLLoader`

        [Ruffle(NativeAccessible)]
        private var _url:String;

        [Ruffle(NativeAccessible)]
        private var _contentType: String = "application/x-www-form-urlencoded"; // ignored

        [Ruffle(NativeAccessible)]
        private var _requestHeaders: Array = [];

        private var _digest:String;

        [Ruffle(NativeAccessible)]
        private var _method:String = URLRequestMethod.GET;

        [Ruffle(NativeAccessible)]
        private var _data:Object;

        public function URLRequest(url:String = null) {
            this._url = url;
        }

        public function get url():String {
            return this._url;
        }

        public function set url(url:String):void {
            this._url = url;
        }

        public function get method():String {
            return this._method;
        }

        public function set method(value: String):void {
            // The method can apparently either be all upper or lower case, but not mixed.
            if (value !== "GET" && value !== "get" && value !== "POST" && value !== "post") {
                throw new ArgumentError("Error #2008: Parameter method must be one of the accepted values.", 2008);
            }

            // TODO: AIR is supposed to support other methods like PUT or DELETE.
            this._method = value;
        }

        public function get data():Object {
            return this._data;
        }

        public function set data(newData:Object):void {
            this._data = newData;
        }

        public function set contentType(value:String):void {
            this._contentType = value;
        }

        public function get contentType():String {
            return this._contentType;
        }

        public function get requestHeaders():Array {
            return this._requestHeaders;
        }

        public function set requestHeaders(headers:Array):void {
            this._requestHeaders = headers;
        }

        public function get digest():String {
            return this._digest;
        }

        public function set digest(value:String):void {
            this._digest = value;
        }

    }
}
