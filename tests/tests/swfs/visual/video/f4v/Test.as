package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.media.Video;
    import flash.events.NetStatusEvent;
    import flash.utils.setTimeout;

     [SWF(width="80", height="80", frameRate="10")]
    public class Test extends Sprite {
        public function Test() {
            super();

            var nc = new NetConnection();
            nc.connect(null);
            var ns = new NetStream(nc);
            ns.client = new Object();
            ns.client.onMetaData = function(metaData:Object):void {};
            var vid = new Video(80, 80);
            addChild(vid);
            vid.attachNetStream(ns);
            ns.play("test.mp4");

            setTimeout(function():void {
                ns.seek(1);
            }, 5000);

            setTimeout(function():void {
                ns.seek(3);
            }, 6000);
        }
    }
}
