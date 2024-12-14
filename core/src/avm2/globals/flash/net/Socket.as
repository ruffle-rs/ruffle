package flash.net {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.IDataInput;
    import flash.utils.IDataOutput;

    import __ruffle__.stub_getter;

    [Ruffle(InstanceAllocator)]
    public class Socket extends EventDispatcher implements IDataOutput, IDataInput {

        public function Socket(host:String = null, port:int = 0) {
            this.timeout = 20000;
            if (host != null) {
                this.connect(host, port);
            }
        }

        public native function connect(host: String, port: int):void;

        public native function get timeout():uint;
        public native function set timeout(value:uint):void;

        public native function close():void;

        public native function get bytesAvailable():uint;

        [API("674")]
        public function get bytesPending():uint {
            stub_getter("flash.net.Socket", "bytesPending");
            return 0;
        }

        public native function get endian():String;
        public native function set endian(value:String):void;

        public native function get connected():Boolean;

        public native function get objectEncoding():uint;
        public native function set objectEncoding(value:uint):void;

        public native function flush():void;

        public native function readBoolean():Boolean;
        public native function readByte():int;
        public native function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void;
        public native function readDouble():Number;
        public native function readFloat():Number;
        public native function readInt():int;
        public native function readMultiByte(length:uint, charSet:String):String;
        public native function readObject():*;
        public native function readShort():int;
        public native function readUnsignedByte():uint;
        public native function readUnsignedInt():uint;
        public native function readUnsignedShort():uint;
        public native function readUTF():String;
        public native function readUTFBytes(length:uint):String;

        public native function writeBoolean(value:Boolean):void;
        public native function writeByte(value:int):void;
        public native function writeBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void;
        public native function writeDouble(value:Number):void;
        public native function writeFloat(value:Number):void;
        public native function writeInt(value:int):void;
        public native function writeMultiByte(value:String, charSet:String):void;
        public native function writeObject(value:*):void;
        public native function writeShort(value:int):void;
        public native function writeUnsignedInt(value:uint):void;
        public native function writeUTF(value:String):void;
        public native function writeUTFBytes(value:String):void;
    }
}
