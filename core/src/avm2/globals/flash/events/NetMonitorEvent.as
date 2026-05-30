package flash.events {
    import flash.net.NetStream;

    [API("672")]
    public class NetMonitorEvent extends Event {
        public static const NET_STREAM_CREATE:String = "netStreamCreate";

        private var _netStream:NetStream;

        public function NetMonitorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, netStream:NetStream = null) {
            super(type, bubbles, cancelable);
            this._netStream = netStream;
        }

        public function get netStream():NetStream {
            return this._netStream;
        }

        override public function clone():Event {
            return new NetMonitorEvent(type, bubbles, cancelable, this.netStream);
        }

        override public function toString():String {
            return formatToString("NetMonitorEvent", "type", "bubbles", "cancelable", "eventPhase", "netStream");
        }
    }
}
