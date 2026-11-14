package flash.net {
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.utils.Endian;
    import flash.utils.IDataInput;
    import flash.utils.ByteArray;
    import flash.events.HTTPStatusEvent;
    import flash.events.IOErrorEvent;
    import flash.events.ProgressEvent;
    import flash.events.SecurityErrorEvent;

    public class URLStream extends EventDispatcher implements IDataInput {
        private var _endian:String = Endian.BIG_ENDIAN;
        private var _objectEncoding:uint = ObjectEncoding.DEFAULT;
        private var _connected:Boolean = false;
        private var _loader:URLLoader = new URLLoader();

        public function URLStream() {
            this._loader.dataFormat = URLLoaderDataFormat.BINARY;
            var self = this;

            this._loader.addEventListener(Event.OPEN, function(e:*):void {
                self.dispatchEvent(new Event(Event.OPEN));
            });
            this._loader.addEventListener(Event.COMPLETE, function(e:*):void {
                if (self._loader.data) {
                    self._loader.data.endian = self._endian;
                    self._loader.data.objectEncoding = self._objectEncoding;
                }
                self.dispatchEvent(new Event(Event.COMPLETE));
            });
            this._loader.addEventListener(IOErrorEvent.IO_ERROR, function(e:*):void {
                self.dispatchEvent(new IOErrorEvent(IOErrorEvent.IO_ERROR));
            });
            this._loader.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(e:*):void {
                self.dispatchEvent(new SecurityErrorEvent(SecurityErrorEvent.SECURITY_ERROR));
            });
            this._loader.addEventListener(ProgressEvent.PROGRESS, function(e:*):void {
                if (self._loader.data) {
                    self._loader.data.endian = self._endian;
                    self._loader.data.objectEncoding = self._objectEncoding;
                }
                self.dispatchEvent(new ProgressEvent(ProgressEvent.PROGRESS, false, false, e.bytesLoaded, e.bytesTotal));
            });
            this._loader.addEventListener(HTTPStatusEvent.HTTP_STATUS, function(e:*):void {
                self.dispatchEvent(new HTTPStatusEvent(HTTPStatusEvent.HTTP_STATUS, false, false, e.status, e.redirected));
            });
        }

        public function get bytesAvailable():uint {
            if (this._loader.data) {
                return this._loader.data.bytesAvailable;
            }
            return 0;
        }

        public function get connected():Boolean {
            return _connected;
        }

        public function get endian():String {
            return _endian;
        }

        public function set endian(value:String):void {
            if (value === Endian.BIG_ENDIAN || value === Endian.LITTLE_ENDIAN) {
                this._endian = value;
                if (this._loader.data) {
                    this._loader.data.endian = value;
                }
            } else {
                throw new ArgumentError("Error #2008: Parameter endian must be one of the accepted values.", 2008);
            }
        }

        public function load(request:URLRequest):void {
            this._loader.load(request);
            this._connected = true;
        }

        public function close():void {
            this._loader.close();
            this._connected = false;
        }

        public function get objectEncoding():uint {
            return this._objectEncoding;
        }

        public function set objectEncoding(value:uint):void {
            if (value !== ObjectEncoding.AMF0 && value !== ObjectEncoding.AMF3) {
                throw new ArgumentError("Error #2008: Parameter objectEncoding must be one of the accepted values.", 2008);
            }
            this._objectEncoding = value;
            if (this._loader.data) {
                this._loader.data.objectEncoding = value;
            }
        }

        public function readBoolean():Boolean {
            return this._loader.data.readBoolean();
        }

        public function readByte():int {
            return this._loader.data.readByte();
        }

        public function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void {
            this._loader.data.readBytes(bytes, offset, length);
        }

        public function readDouble():Number {
            return this._loader.data.readDouble();
        }

        public function readFloat():Number {
            return this._loader.data.readFloat();
        }

        public function readInt():int {
            return this._loader.data.readInt();
        }

        public function readMultiByte(length:uint, charSet:String):String {
            return this._loader.data.readMultiByte(length, charSet);
        }

        public function readObject():* {
            return this._loader.data.readObject();
        }

        public function readShort():int {
            return this._loader.data.readShort();
        }

        public function readUnsignedByte():uint {
            return this._loader.data.readUnsignedByte();
        }

        public function readUnsignedInt():uint {
            return this._loader.data.readUnsignedInt();
        }

        public function readUnsignedShort():uint {
            return this._loader.data.readUnsignedShort();
        }

        public function readUTF():String {
            return this._loader.data.readUTF();
        }

        public function readUTFBytes(length:uint):String {
            return this._loader.data.readUTFBytes(length);
        }
    }
}
