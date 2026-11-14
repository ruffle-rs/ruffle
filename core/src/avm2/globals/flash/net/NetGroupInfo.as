package flash.net {
    [API("667")]
    public final class NetGroupInfo {
        private var _postingSendDataBytesPerSecond:Number;
        private var _postingSendControlBytesPerSecond:Number;
        private var _postingReceiveDataBytesPerSecond:Number;
        private var _postingReceiveControlBytesPerSecond:Number;
        private var _routingSendBytesPerSecond:Number;
        private var _routingReceiveBytesPerSecond:Number;
        private var _objectReplicationSendBytesPerSecond:Number;
        private var _objectReplicationReceiveBytesPerSecond:Number;

        public function NetGroupInfo(
            postingSendDataBytesPerSecond:Number = 0,
            postingSendControlBytesPerSecond:Number = 0,
            postingReceiveDataBytesPerSecond:Number = 0,
            postingReceiveControlBytesPerSecond:Number = 0,
            routingSendBytesPerSecond:Number = 0,
            routingReceiveBytesPerSecond:Number = 0,
            objectReplicationSendBytesPerSecond:Number = 0,
            objectReplicationReceiveBytesPerSecond:Number = 0
        ) {
            this._postingSendDataBytesPerSecond = postingSendDataBytesPerSecond;
            this._postingSendControlBytesPerSecond = postingSendControlBytesPerSecond;
            this._postingReceiveDataBytesPerSecond = postingReceiveDataBytesPerSecond;
            this._postingReceiveControlBytesPerSecond = postingReceiveControlBytesPerSecond;
            this._routingSendBytesPerSecond = routingSendBytesPerSecond;
            this._routingReceiveBytesPerSecond = routingReceiveBytesPerSecond;
            this._objectReplicationSendBytesPerSecond = objectReplicationSendBytesPerSecond;
            this._objectReplicationReceiveBytesPerSecond = objectReplicationReceiveBytesPerSecond;
        }

        public function get postingSendDataBytesPerSecond():Number {
            return this._postingSendDataBytesPerSecond;
        }

        public function get postingSendControlBytesPerSecond():Number {
            return this._postingSendControlBytesPerSecond;
        }

        public function get postingReceiveDataBytesPerSecond():Number {
            return this._postingReceiveDataBytesPerSecond;
        }

        public function get postingReceiveControlBytesPerSecond():Number {
            return this._postingReceiveControlBytesPerSecond;
        }

        public function get routingSendBytesPerSecond():Number {
            return this._routingSendBytesPerSecond;
        }

        public function get routingReceiveBytesPerSecond():Number {
            return this._routingReceiveBytesPerSecond;
        }

        public function get objectReplicationSendBytesPerSecond():Number {
            return this._objectReplicationSendBytesPerSecond;
        }

        public function get objectReplicationReceiveBytesPerSecond():Number {
            return this._objectReplicationReceiveBytesPerSecond;
        }

        public function toString():String {
            return "postingSendDataBytesPerSecond=" + this._postingSendDataBytesPerSecond +
                " postingSendControlBytesPerSecond=" + this._postingSendControlBytesPerSecond +
                " postingReceiveDataBytesPerSecond=" + this._postingReceiveDataBytesPerSecond +
                " postingReceiveControlBytesPerSecond=" + this._postingReceiveControlBytesPerSecond +
                " routingSendBytesPerSecond=" + this._routingSendBytesPerSecond +
                " routingReceiveBytesPerSecond=" + this._routingReceiveBytesPerSecond +
                " objectReplicationSendBytesPerSecond=" + this._objectReplicationSendBytesPerSecond +
                " objectReplicationReceiveBytesPerSecond=" + this._objectReplicationReceiveBytesPerSecond;
        }
    }
}
