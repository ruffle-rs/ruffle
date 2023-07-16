package flash.net {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.IDataInput;
    import flash.utils.IDataOutput;

    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    [Ruffle(InstanceAllocator)]
    public class Socket extends EventDispatcher implements IDataOutput, IDataInput {
        private var _timeout:uint;

        public function Socket(host:String = null, port:int = 0) {
            this._timeout = 20000;
            if (host != null) {
                this.connect(host, port);
            }
        }

        public native function connect(host: String, port: int):void;


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

        public native function close():void;

        public native function get bytesAvailable():uint;

        public function get bytesPending():uint {
            stub_getter("flash.net.Socket", "bytesPending");
            return 0;
        }

        public native function get endian():String;
        public native function set endian(value:String):void;

        public native function get connected():Boolean;

        public function get objectEncoding():uint {
            stub_getter("flash.net.Socket", "objectEncoding");
            return 0;
        }

        public function set objectEncoding(value:uint):void {
            stub_setter("flash.net.Socket", "objectEncoding");
        }

        public native function flush():void;

        public native function readBoolean():Boolean;
        public native function readByte():int;
        public native function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void;
        public native function readDouble():Number;
        public native function readFloat():Number;
        public native function readInt():int;

        public function readMultiByte(length:uint, charSet:String):String {
            stub_method("flash.net.Socket", "readMultiByte");
            return "";
        }

        public function readObject():* {
            stub_method("flash.net.Socket", "readObject");
            return null;
        }

        public native function readShort():int;
        public native function readUnsignedByte():uint;
        public native function readUnsignedInt():uint;
        public native function readUnsignedShort():uint;

        public function readUTF():String {
            stub_method("flash.net.Socket", "readUTF");
            return "";
        }

        public function readUTFBytes(length:uint):String {
            stub_method("flash.net.Socket", "readUTFBytes");
            return "";
        }

        public native function writeBoolean(value:Boolean):void;
        public native function writeByte(value:int):void;
        public native function writeBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void;
        public native function writeDouble(value:Number):void;
        public native function writeFloat(value:Number):void;
        public native function writeInt(value:int):void;

        public function writeMultiByte(value:String, charSet:String):void {
            stub_method("flash.net.Socket", "writeMultiByte");
        }

        public function writeObject(value:*):void {
            stub_method("flash.net.Socket", "writeObject");
        }

        public native function writeShort(value:int):void;
        public native function writeUnsignedInt(value:uint):void;

        public function writeUTF(value:String):void {
            stub_method("flash.net.Socket", "writeUTF");
        }

        public function writeUTFBytes(value:String):void {
            stub_method("flash.net.Socket", "writeUTFBytes");
        }
    }
}
