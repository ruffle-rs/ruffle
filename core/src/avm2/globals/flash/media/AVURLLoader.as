package flash.media {
    import flash.events.EventDispatcher;
    import flash.net.URLRequest;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;

    [API("688")]
    public class AVURLLoader extends EventDispatcher {
        private var _loader:URLLoader;
        private var _dataFormat:String = URLLoaderDataFormat.TEXT;

        public function AVURLLoader(request:URLRequest = null) {
            super();
            this._loader = new URLLoader();
            if (request != null) {
                this.load(request);
            }
        }

        public function get data():* {
            return this._loader.data;
        }

        public function get dataFormat():String {
            return this._dataFormat;
        }
        public function set dataFormat(value:String):void {
            this._dataFormat = value;
            this._loader.dataFormat = value;
        }

        public function load(request:URLRequest):void {
            this._loader.load(request);
        }

        public function close():void {
            this._loader.close();
        }
    }
}
