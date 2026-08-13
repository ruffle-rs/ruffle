package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Video;
    import flash.net.NetConnection;
    import flash.net.NetStream;

    public class Test extends Sprite {
        private var stream:NetStream;
        private var framesAfterMetadata:uint;

        public function Test() {
            var connection:NetConnection = new NetConnection();
            connection.connect(null);

            stream = new NetStream(connection);
            stream.client = { onMetaData: onMetaData };

            var video:Video = new Video(320, 233);
            video.attachNetStream(stream);
            addChild(video);

            stream.play("test_video.flv");
            stream.pause();
        }

        private function onMetaData(info:Object):void {
            trace("onMetaData");
            trace("paused time: " + stream.time);
            addEventListener(Event.ENTER_FRAME, onEnterFrame);
        }

        private function onEnterFrame(event:Event):void {
            framesAfterMetadata++;
            if (framesAfterMetadata == 10) {
                trace("later paused time: " + stream.time);
                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
            }
        }
    }
}
