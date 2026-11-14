package flash.net {
    import flash.events.EventDispatcher;

    [API("672")]
    public class NetMonitor extends EventDispatcher {
        public function NetMonitor() {
            super();
        }

        public function listStreams():Vector.<NetStream> {
            return new Vector.<NetStream>();
        }
    }
}
