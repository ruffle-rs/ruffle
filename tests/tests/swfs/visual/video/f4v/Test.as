package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.media.Video;
    import flash.events.NetStatusEvent;

     [SWF(width="400", height="220")]
    public class Test extends Sprite {
        public function Test() {
            super();

            var nc = new NetConnection();
            nc.connect(null);
            var ns = new NetStream(nc);
            ns.client = new Object();
            ns.client.onMetaData = function(metaData:Object):void {};
            var vid = new Video(400, 220);
            addChild(vid);
            vid.attachNetStream(ns);
            ns.play("logo-anim.f4v");
        }
    }
}