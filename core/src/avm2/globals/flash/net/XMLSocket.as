package flash.net
{
    import flash.net.Socket;
    import flash.events.EventDispatcher;
    import flash.events.ProgressEvent;
    import flash.events.IOErrorEvent;
    import flash.events.SecurityErrorEvent;
    import flash.events.Event;
    import flash.events.DataEvent;
    import flash.utils.ByteArray;

    public class XMLSocket extends EventDispatcher
    {
        private var tempBuf:ByteArray = new ByteArray();
        private var socket:Socket;

        public function XMLSocket(host:String = null, port:int = 0)
        {
            this.socket = new Socket();

            this.socket.addEventListener(Event.CLOSE, this.socketCloseListener);
            this.socket.addEventListener(Event.CONNECT, this.socketConnectEvent);
            this.socket.addEventListener(ProgressEvent.SOCKET_DATA, this.socketDataListener);
            this.socket.addEventListener(IOErrorEvent.IO_ERROR, this.socketIoErrorListener);
            this.socket.addEventListener(SecurityErrorEvent.SECURITY_ERROR, this.socketSecurityErrorListener);

            if (host != null)
            {
                this.connect(host, port);
            }
        }

        private native function get domain():String;

        private function socketCloseListener(evt:Event):void
        {
            this.tempBuf.clear();
            this.dispatchEvent(evt);
        }

        private function socketConnectEvent(evt:Event):void
        {
            this.dispatchEvent(evt);
        }

        private function socketDataListener(evt:ProgressEvent):void
        {
            // FIXME: There is probably a better way to do this.
            for (var i:uint = 0; i < evt.bytesLoaded; i++)
            {
                var byte:int = this.socket.readByte();

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

        private function socketIoErrorListener(evt:IOErrorEvent):void
        {
            this.dispatchEvent(evt);
        }

        private function socketSecurityErrorListener(evt:SecurityErrorEvent):void
        {
            this.dispatchEvent(evt);
        }

        public function get connected():Boolean
        {
            return this.socket.connected;
        }

        public function get timeout():int
        {
            return this.socket.timeout;
        }

        public function set timeout(value:int)
        {
            this.socket.timeout = value;
        }

        public function close():void
        {
            this.tempBuf.clear();
            this.socket.close();
        }

        public function connect(host:String, port:int):void
        {
            if (host == null)
            {
                host = this.domain();
            }

            socket.connect(host, port);
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

            this.socket.writeUTFBytes(val);
            this.socket.writeByte(0);
            this.socket.flush();
        }
    }
}
