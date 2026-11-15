package flash.filesystem {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.IDataInput;
    import flash.utils.IDataOutput;

    [API("661")]
    public class FileStream extends EventDispatcher implements IDataInput, IDataOutput {
        private var _buffer:ByteArray = new ByteArray();
        private var _endian:String = Endian.BIG_ENDIAN;
        private var _objectEncoding:uint = 3;

        public function FileStream() {
            super();
        }

        public function open(file:File, fileMode:String):void {
            this._buffer.position = 0;
        }

        public function openAsync(file:File, fileMode:String):void {
            this._buffer.position = 0;
        }

        public function close():void {
            this._buffer.clear();
        }

        public function truncate():void {
            this._buffer.length = this._buffer.position;
        }

        public function get position():Number {
            return this._buffer.position;
        }
        public function set position(value:Number):void {
            this._buffer.position = uint(value);
        }

        public function get bytesAvailable():uint {
            return this._buffer.bytesAvailable;
        }

        public function get endian():String {
            return this._endian;
        }
        public function set endian(value:String):void {
            this._endian = value;
            this._buffer.endian = value;
        }

        public function get objectEncoding():uint {
            return this._objectEncoding;
        }
        public function set objectEncoding(value:uint):void {
            this._objectEncoding = value;
            this._buffer.objectEncoding = value;
        }

        public function readBoolean():Boolean { return this._buffer.readBoolean(); }
        public function readByte():int { return this._buffer.readByte(); }
        public function readBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void { this._buffer.readBytes(bytes, offset, length); }
        public function readDouble():Number { return this._buffer.readDouble(); }
        public function readFloat():Number { return this._buffer.readFloat(); }
        public function readInt():int { return this._buffer.readInt(); }
        public function readMultiByte(length:uint, charSet:String):String { return this._buffer.readMultiByte(length, charSet); }
        public function readObject():* { return this._buffer.readObject(); }
        public function readShort():int { return this._buffer.readShort(); }
        public function readUnsignedByte():uint { return this._buffer.readUnsignedByte(); }
        public function readUnsignedInt():uint { return this._buffer.readUnsignedInt(); }
        public function readUnsignedShort():uint { return this._buffer.readUnsignedShort(); }
        public function readUTF():String { return this._buffer.readUTF(); }
        public function readUTFBytes(length:uint):String { return this._buffer.readUTFBytes(length); }

        public function writeBoolean(value:Boolean):void { this._buffer.writeBoolean(value); }
        public function writeByte(value:int):void { this._buffer.writeByte(value); }
        public function writeBytes(bytes:ByteArray, offset:uint = 0, length:uint = 0):void { this._buffer.writeBytes(bytes, offset, length); }
        public function writeDouble(value:Number):void { this._buffer.writeDouble(value); }
        public function writeFloat(value:Number):void { this._buffer.writeFloat(value); }
        public function writeInt(value:int):void { this._buffer.writeInt(value); }
        public function writeMultiByte(value:String, charSet:String):void { this._buffer.writeMultiByte(value, charSet); }
        public function writeObject(object:*):void { this._buffer.writeObject(object); }
        public function writeShort(value:int):void { this._buffer.writeShort(value); }
        public function writeUnsignedInt(value:uint):void { this._buffer.writeUnsignedInt(value); }
        public function writeUTF(value:String):void { this._buffer.writeUTF(value); }
        public function writeUTFBytes(value:String):void { this._buffer.writeUTFBytes(value); }
    }
}
