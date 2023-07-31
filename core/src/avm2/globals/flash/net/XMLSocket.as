package flash.net
{
    import flash.net.Socket;
    import flash.events.ProgressEvent;
    import flash.events.DataEvent;
    import flash.utils.ByteArray;

    // NOTE: Extending from Socket is a implementation detail. It is not possible for SWFs to
    // use Socket methods as Flash's playerglobal doesn't include those methods, unless
    // SWF author intentionally tried, which wouldn't make much sense.
    public class XMLSocket extends Socket
    {
        // Events ioError, connect, close, securityError are inherited from Socket.
        // connected, timeout are inherited from Socket.

        private var tempBuf:ByteArray = new ByteArray();

        public function XMLSocket(host:String = null, port:int = 0)
        {
            super(null, 0);

            this.addEventListener(ProgressEvent.SOCKET_DATA, this.socketDataListener);

            if (host != null)
            {
                this.connect(host, port);
            }
        }

        private native function get domain():String;

        override public function connect(host:String, port:int):void
        {
            if (host == null)
            {
                host = this.domain();
            }

            super.connect(host, port);
        }

        function socketDataListener(evt:ProgressEvent):void
        {
            // FIXME: There is probably a better way to do this.
            for (var i:uint = 0; i < evt.bytesLoaded; i++)
            {
                var byte:int = this.readByte();

                if (byte == 0)
                {
                    var length:uint = this.tempBuf.position;
                    this.tempBuf.position = 0;

                    var data:String = this.tempBuf.readUTFBytes(length);
                    this.tempBuf.clear();

                    this.dispatchEvent(new DataEvent(DataEvent.DATA, false, false, data));
                }
                else
                {
                    this.tempBuf.writeByte(byte);
                }
            }
        }

        public function send(object:*):void
        {
            var val:String;

            if (object is XML || object is XMLList)
            {
                val = object.toXMLString();
            }
            else
            {
                val = object.toString();
            }

            this.writeUTFBytes(val);
            this.writeByte(0);
            this.flush();
        }
    }
}
