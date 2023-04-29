package flash.net {
    import flash.events.EventDispatcher;
    import flash.utils.Endian;
    
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    public class Socket extends EventDispatcher {
        private var _timeout:uint;
        
        private var _endian:String = Endian.BIG_ENDIAN;

        public function Socket(host:String = null, port:int = 0) {
            this._timeout = 20000;
            if (host != null) {
                this.connect(host, port);
            }
        }

        public function connect(host: String, port: int):void {
            stub_method("flash.net.Socket", "connect");
        }


        public function get timeout():uint {
            return this._timeout;
        }

        public function set timeout(value:uint):void {
            if (value < 250) {
                this._timeout = 250;
            } else {
                this._timeout = value;
            }
        }

        public function close():void {
            stub_method("flash.net.Socket", "close");
        }

        public function get bytesPending():uint {
            stub_getter("flash.net.Socket", "bytesPending");
            return 0;
        }
        
        public function get endian():String {
            return this._endian;
        }
        
        public function set endian(value:String) {
            if (value === Endian.BIG_ENDIAN || value === Endian.LITTLE_ENDIAN) {
                this._endian = value;
            } else {
                throw new ArgumentError("Error #2008: Parameter endian must be one of the accepted values.", 2008);
            }
        }
    }
}
