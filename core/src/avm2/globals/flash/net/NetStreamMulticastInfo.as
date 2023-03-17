package flash.net {
    public final class NetStreamMulticastInfo {
        private var _bytesPushedFromPeers : Number;
        private var _bytesPushedToPeers : Number;
        private var _bytesReceivedFromIPMulticast : Number;
        private var _bytesReceivedFromServer : Number;
        private var _bytesRequestedByPeers : Number;
        private var _bytesRequestedFromPeers : Number;
        private var _fragmentsPushedFromPeers : Number;
        private var _fragmentsPushedToPeers : Number;
        private var _fragmentsReceivedFromIPMulticast : Number;
        private var _fragmentsReceivedFromServer : Number;
        private var _fragmentsRequestedByPeers : Number;
        private var _fragmentsRequestedFromPeers : Number;
        private var _receiveControlBytesPerSecond : Number;
        private var _receiveDataBytesPerSecond : Number;
        private var _receiveDataBytesPerSecondFromIPMulticast : Number;
        private var _receiveDataBytesPerSecondFromServer : Number;
        private var _sendControlBytesPerSecond : Number;
        private var _sendControlBytesPerSecondToServer : Number;
        private var _sendDataBytesPerSecond : Number;

        public function toString():String {
            __ruffle__.stub_method("flash.net.NetStreamMulticastInfo", "toString");

            return "";
        }

        public function get bytesPushedFromPeers() : Number {
            return this._bytesPushedFromPeers;
        };

        public function get bytesPushedToPeers() : Number {
            return this._bytesPushedToPeers;
        };

        public function get bytesReceivedFromIPMulticast() : Number {
            return this._bytesReceivedFromIPMulticast;
        };

        public function get bytesReceivedFromServer() : Number {
            return this._bytesReceivedFromServer;
        };

        public function get bytesRequestedByPeers() : Number {
            return this._bytesRequestedByPeers;
        };

        public function get bytesRequestedFromPeers() : Number {
            return this._bytesRequestedFromPeers;
        };

        public function get fragmentsPushedFromPeers() : Number {
            return this._fragmentsPushedFromPeers;
        };

        public function get fragmentsPushedToPeers() : Number {
            return this._fragmentsPushedToPeers;
        };

        public function get fragmentsReceivedFromIPMulticast() : Number {
            return this._fragmentsReceivedFromIPMulticast;
        };

        public function get fragmentsReceivedFromServer() : Number {
            return this._fragmentsReceivedFromServer;
        };

        public function get fragmentsRequestedByPeers() : Number {
            return this._fragmentsRequestedByPeers;
        };

        public function get fragmentsRequestedFromPeers() : Number {
            return this._fragmentsRequestedFromPeers;
        };

        public function get receiveControlBytesPerSecond() : Number {
            return this._receiveControlBytesPerSecond;
        };

        public function get receiveDataBytesPerSecond() : Number {
            return this._receiveDataBytesPerSecond;
        };

        public function get receiveDataBytesPerSecondFromIPMulticast() : Number {
            return this._receiveDataBytesPerSecondFromIPMulticast;
        };

        public function get receiveDataBytesPerSecondFromServer() : Number {
            return this._receiveDataBytesPerSecondFromServer;
        };

        public function get sendControlBytesPerSecond() : Number {
            return this._sendControlBytesPerSecond;
        };

        public function get sendControlBytesPerSecondToServer() : Number {
            return this._sendControlBytesPerSecondToServer;
        };

        public function get sendDataBytesPerSecond() : Number {
            return this._sendDataBytesPerSecond;
        };

    }
}