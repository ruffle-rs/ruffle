package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.events.NetStatusEvent;

    public class Test extends Sprite {
        private var nc:NetConnection;
        private var ns:NetStream;
        private var replayed:Boolean = false;

        public function Test() {
            nc = new NetConnection();
            nc.connect(null);
            
            ns = new NetStream(nc);
            ns.addEventListener(NetStatusEvent.NET_STATUS, onStatus);
            ns.client = {
                onMetaData: function(info:Object):void {}
            };
            
            trace("Playing video first time");
            ns.play("test.flv");
        }

        private function onStatus(event:NetStatusEvent):void {
            trace(event.info.code);
            if (event.info.code == "NetStream.Play.Stop" && !replayed) {
                replayed = true;
                trace("Replaying video");
                ns.play("test.flv");
            }
        }
    }
}
