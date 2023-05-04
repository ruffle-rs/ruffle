package flash.net {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.IDataInput;
    import flash.utils.IDataOutput;

    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public class Socket extends EventDispatcher implements IDataOutput, IDataInput {
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

        public function get bytesAvailable():uint {
            stub_getter("flash.net.Socket", "bytesAvailable");
            return 0;
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

        public function get connected():Boolean {
            stub_getter("flash.net.Socket", "connected");
            return false;
        }

        public function get objectEncoding():uint {
            stub_getter("flash.net.Socket", "objectEncoding");
            return 0;
        }

        public function set objectEncoding(value:uint):void {
            stub_setter("flash.net.Socket", "objectEncoding");
        }

        public function flush():void {
            stub_method("flash.net.Socket", "flush");
        }

        public function readBoolean():Boolean {
            stub_method("flash.net.Socket", "readBoolean");
            return false;
        }

        public function readByte():int {
            stub_method("flash.net.Socket", "readByte");
            return 0;
        }

        public function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void {
            stub_method("flash.net.Socket", "readBytes");
        }

        public function readDouble():Number {
            stub_method("flash.net.Socket", "readDouble");
            return 0.0;
        }

        public function readFloat():Number {
            stub_method("flash.net.Socket", "readFloat");
            return 0.0;
        }

        public function readInt():int {
            stub_method("flash.net.Socket", "readInt");
            return 0;
        }

        public function readMultiByte(length:uint, charSet:String):String {
            stub_method("flash.net.Socket", "readMultiByte");
            return "";
        }

        public function readObject():* {
            stub_method("flash.net.Socket", "readObject");
            return null;
        }

        public function readShort():int {
            stub_method("flash.net.Socket", "readShort");
            return 0;
        }

        public function readUnsignedByte():uint {
            stub_method("flash.net.Socket", "readUnsignedByte");
            return 0;
        }

        public function readUnsignedInt():uint {
            stub_method("flash.net.Socket", "readUnsignedInt");
            return 0;
        }

        public function readUnsignedShort():uint {
            stub_method("flash.net.Socket", "readUnsignedShort");
            return 0;
        }

        public function readUTF():String {
            stub_method("flash.net.Socket", "readUTF");
            return "";
        }

        public function readUTFBytes(length:uint):String {
            stub_method("flash.net.Socket", "readUTFBytes");
            return "";
        }

        public function writeBoolean(value:Boolean):void {
            stub_method("flash.net.Socket", "writeBoolean");
        }

        public function writeByte(value:int):void {
            stub_method("flash.net.Socket", "writeByte");
        }

        public function writeBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void {
            stub_method("flash.net.Socket", "writeBytes");
        }

        public function writeDouble(value:Number):void {
            stub_method("flash.net.Socket", "writeDouble");
        }

        public function writeFloat(value:Number):void {
            stub_method("flash.net.Socket", "writeFloat");
        }

        public function writeInt(value:int):void {
            stub_method("flash.net.Socket", "writeInt");
        }

        public function writeMultiByte(value:String, charSet:String):void {
            stub_method("flash.net.Socket", "writeMultiByte");
        }

        public function writeObject(value:*):void {
            stub_method("flash.net.Socket", "writeObject");
        }

        public function writeShort(value:int):void {
            stub_method("flash.net.Socket", "writeShort");
        }

        public function writeUnsignedInt(value:uint):void {
            stub_method("flash.net.Socket", "writeUnsignedInt");
        }

        public function writeUTF(value:String):void {
            stub_method("flash.net.Socket", "writeString");
        }

        public function writeUTFBytes(value:String):void {
            stub_method("flash.net.Socket", "writeUTFBytes");
        }
    }
}
