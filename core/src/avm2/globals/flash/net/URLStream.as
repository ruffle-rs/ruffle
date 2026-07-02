package flash.net {
    import flash.events.EventDispatcher;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.IDataInput;

    public class URLStream extends EventDispatcher implements IDataInput {
        // The internal byte buffer fed by the native loader. All reads go
        // through this ByteArray. Native code appends bytes to it as they
        // arrive over the network.
        [Ruffle(NativeAccessible)]
        private var _data:ByteArray = new ByteArray();

        [Ruffle(NativeAccessible)]
        private var _connected:Boolean = false;

        // Set to true by close(); the native fetch loop polls this flag and
        // stops dispatching further events once it becomes true.
        [Ruffle(NativeAccessible)]
        private var _closed:Boolean = false;

        public function URLStream() {
        }

        public function get bytesAvailable():uint {
            return this._data.bytesAvailable;
        }

        public function get connected():Boolean {
            return this._connected;
        }

        public function get endian():String {
            return this._data.endian;
        }

        public function set endian(value:String):void {
            if (value === Endian.BIG_ENDIAN || value === Endian.LITTLE_ENDIAN) {
                this._data.endian = value;
            } else {
                throw new ArgumentError("Error #2008: Parameter endian must be one of the accepted values.", 2008);
            }
        }

        public function get objectEncoding():uint {
            return this._data.objectEncoding;
        }

        public function set objectEncoding(value:uint):void {
            this._data.objectEncoding = value;
        }

        public native function load(request:URLRequest):void;
        public native function close():void;

        public function readBoolean():Boolean {
            return this._data.readBoolean();
        }

        public function readByte():int {
            return this._data.readByte();
        }

        public function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void {
            this._data.readBytes(bytes, offset, length);
        }

        public function readDouble():Number {
            return this._data.readDouble();
        }

        public function readFloat():Number {
            return this._data.readFloat();
        }

        public function readInt():int {
            return this._data.readInt();
        }

        public function readMultiByte(length:uint, charSet:String):String {
            return this._data.readMultiByte(length, charSet);
        }

        public function readObject():* {
            return this._data.readObject();
        }

        public function readShort():int {
            return this._data.readShort();
        }

        public function readUnsignedByte():uint {
            return this._data.readUnsignedByte();
        }

        public function readUnsignedInt():uint {
            return this._data.readUnsignedInt();
        }

        public function readUnsignedShort():uint {
            return this._data.readUnsignedShort();
        }

        public function readUTF():String {
            return this._data.readUTF();
        }

        public function readUTFBytes(length:uint):String {
            return this._data.readUTFBytes(length);
        }
    }
}
