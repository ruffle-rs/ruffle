package flash.net {
    public final class NetStreamMulticastInfo {
        private var _bytesPushedFromPeers:Number;
        private var _bytesPushedToPeers:Number;
        private var _bytesReceivedFromIPMulticast:Number;
        private var _bytesReceivedFromServer:Number;
        private var _bytesRequestedByPeers:Number;
        private var _bytesRequestedFromPeers:Number;
        private var _fragmentsPushedFromPeers:Number;
        private var _fragmentsPushedToPeers:Number;
        private var _fragmentsReceivedFromIPMulticast:Number;
        private var _fragmentsReceivedFromServer:Number;
        private var _fragmentsRequestedByPeers:Number;
        private var _fragmentsRequestedFromPeers:Number;
        private var _receiveControlBytesPerSecond:Number;
        private var _receiveDataBytesPerSecond:Number;
        private var _receiveDataBytesPerSecondFromIPMulticast:Number;
        private var _receiveDataBytesPerSecondFromServer:Number;
        private var _sendControlBytesPerSecond:Number;
        private var _sendControlBytesPerSecondToServer:Number;
        private var _sendDataBytesPerSecond:Number;

        public function NetStreamMulticastInfo(
            sendDataBytesPerSecond:Number = 0,
            sendControlBytesPerSecond:Number = 0,
            receiveDataBytesPerSecond:Number = 0,
            receiveControlBytesPerSecond:Number = 0,
            bytesPushedToPeers:Number = 0,
            fragmentsPushedToPeers:Number = 0,
            bytesRequestedByPeers:Number = 0,
            fragmentsRequestedByPeers:Number = 0,
            bytesPushedFromPeers:Number = 0,
            fragmentsPushedFromPeers:Number = 0,
            bytesRequestedFromPeers:Number = 0,
            fragmentsRequestedFromPeers:Number = 0,
            sendControlBytesPerSecondToServer:Number = 0,
            receiveDataBytesPerSecondFromServer:Number = 0,
            bytesReceivedFromServer:Number = 0,
            fragmentsReceivedFromServer:Number = 0,
            receiveDataBytesPerSecondFromIPMulticast:Number = 0,
            bytesReceivedFromIPMulticast:Number = 0,
            fragmentsReceivedFromIPMulticast:Number = 0
        ) {
            this._sendDataBytesPerSecond = sendDataBytesPerSecond;
            this._sendControlBytesPerSecond = sendControlBytesPerSecond;
            this._receiveDataBytesPerSecond = receiveDataBytesPerSecond;
            this._receiveControlBytesPerSecond = receiveControlBytesPerSecond;
            this._bytesPushedToPeers = bytesPushedToPeers;
            this._fragmentsPushedToPeers = fragmentsPushedToPeers;
            this._bytesRequestedByPeers = bytesRequestedByPeers;
            this._fragmentsRequestedByPeers = fragmentsRequestedByPeers;
            this._bytesPushedFromPeers = bytesPushedFromPeers;
            this._fragmentsPushedFromPeers = fragmentsPushedFromPeers;
            this._bytesRequestedFromPeers = bytesRequestedFromPeers;
            this._fragmentsRequestedFromPeers = fragmentsRequestedFromPeers;
            this._sendControlBytesPerSecondToServer = sendControlBytesPerSecondToServer;
            this._receiveDataBytesPerSecondFromServer = receiveDataBytesPerSecondFromServer;
            this._bytesReceivedFromServer = bytesReceivedFromServer;
            this._fragmentsReceivedFromServer = fragmentsReceivedFromServer;
            this._receiveDataBytesPerSecondFromIPMulticast = receiveDataBytesPerSecondFromIPMulticast;
            this._bytesReceivedFromIPMulticast = bytesReceivedFromIPMulticast;
            this._fragmentsReceivedFromIPMulticast = fragmentsReceivedFromIPMulticast;
        }

        public function toString():String {
            return "sendDataBytesPerSecond=" + this._sendDataBytesPerSecond +
                " sendControlBytesPerSecond=" + this._sendControlBytesPerSecond +
                " receiveDataBytesPerSecond=" + this._receiveDataBytesPerSecond +
                " receiveControlBytesPerSecond=" + this._receiveControlBytesPerSecond +
                " bytesPushedToPeers=" + this._bytesPushedToPeers +
                " fragmentsPushedToPeers=" + this._fragmentsPushedToPeers +
                " bytesRequestedByPeers=" + this._bytesRequestedByPeers +
                " fragmentsRequestedByPeers=" + this._fragmentsRequestedByPeers +
                " bytesPushedFromPeers=" + this._bytesPushedFromPeers +
                " fragmentsPushedFromPeers=" + this._fragmentsPushedFromPeers +
                " bytesRequestedFromPeers=" + this._bytesRequestedFromPeers +
                " fragmentsRequestedFromPeers=" + this._fragmentsRequestedFromPeers +
                " sendControlBytesPerSecondToServer=" + this._sendControlBytesPerSecondToServer +
                " receiveDataBytesPerSecondFromServer=" + this._receiveDataBytesPerSecondFromServer +
                " bytesReceivedFromServer=" + this._bytesReceivedFromServer +
                " fragmentsReceivedFromServer=" + this._fragmentsReceivedFromServer +
                " receiveDataBytesPerSecondFromIPMulticast=" + this._receiveDataBytesPerSecondFromIPMulticast +
                " bytesReceivedFromIPMulticast=" + this._bytesReceivedFromIPMulticast +
                " fragmentsReceivedFromIPMulticast=" + this._fragmentsReceivedFromIPMulticast;
        }

        public function get bytesPushedFromPeers():Number {
            return this._bytesPushedFromPeers;
        }

        public function get bytesPushedToPeers():Number {
            return this._bytesPushedToPeers;
        }

        public function get bytesReceivedFromIPMulticast():Number {
            return this._bytesReceivedFromIPMulticast;
        }

        public function get bytesReceivedFromServer():Number {
            return this._bytesReceivedFromServer;
        }

        public function get bytesRequestedByPeers():Number {
            return this._bytesRequestedByPeers;
        }

        public function get bytesRequestedFromPeers():Number {
            return this._bytesRequestedFromPeers;
        }

        public function get fragmentsPushedFromPeers():Number {
            return this._fragmentsPushedFromPeers;
        }

        public function get fragmentsPushedToPeers():Number {
            return this._fragmentsPushedToPeers;
        }

        public function get fragmentsReceivedFromIPMulticast():Number {
            return this._fragmentsReceivedFromIPMulticast;
        }

        public function get fragmentsReceivedFromServer():Number {
            return this._fragmentsReceivedFromServer;
        }

        public function get fragmentsRequestedByPeers():Number {
            return this._fragmentsRequestedByPeers;
        }

        public function get fragmentsRequestedFromPeers():Number {
            return this._fragmentsRequestedFromPeers;
        }

        public function get receiveControlBytesPerSecond():Number {
            return this._receiveControlBytesPerSecond;
        }

        public function get receiveDataBytesPerSecond():Number {
            return this._receiveDataBytesPerSecond;
        }

        public function get receiveDataBytesPerSecondFromIPMulticast():Number {
            return this._receiveDataBytesPerSecondFromIPMulticast;
        }

        public function get receiveDataBytesPerSecondFromServer():Number {
            return this._receiveDataBytesPerSecondFromServer;
        }

        public function get sendControlBytesPerSecond():Number {
            return this._sendControlBytesPerSecond;
        }

        public function get sendControlBytesPerSecondToServer():Number {
            return this._sendControlBytesPerSecondToServer;
        }

        public function get sendDataBytesPerSecond():Number {
            return this._sendDataBytesPerSecond;
        }
    }
}
