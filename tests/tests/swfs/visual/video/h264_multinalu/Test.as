package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.media.Video;

    [SWF(width="432", height="432", frameRate="8")]
    public class Test extends Sprite {
        public function Test() {
            super();

            var nc = new NetConnection();
            nc.connect(null);
            var ns = new NetStream(nc);
            ns.client = new Object();
            ns.client.onMetaData = function(metaData:Object):void {};
            var vid = new Video(432, 432);
            addChild(vid);
            vid.attachNetStream(ns);
            ns.play("test.flv");
        }
    }
}
