package
{
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.utils.ByteArray;
    import flash.events.Event;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.net.URLRequest;

    public class Test extends Sprite
    {
        public function Test()
        {
            var connection:NetConnection = new NetConnection();
            connection.connect(null);

            testValidAppend(connection);
            testWithoutPlay(connection);
            testAfterPlayUrl(connection);
            testNull(connection);
            testReentrantAppend(connection);
        }

        private function testValidAppend(connection:NetConnection):void
        {
            trace("valid append");

            var stream:NetStream = new NetStream(connection);
            stream.play(null);

            var bytes:ByteArray = new ByteArray();
            bytes.writeByte(1);
            bytes.writeByte(2);
            bytes.writeByte(3);
            bytes.writeByte(4);

            trace("before length=" + bytes.length + ", position=" + bytes.position);

            bytes.position = 2;
            trace("before append position=" + bytes.position);

            var result:* = stream.appendBytes(bytes);

            trace("result=" + result);
            trace("after length=" + bytes.length + ", position=" + bytes.position);
        }

        private function testWithoutPlay(connection:NetConnection):void
        {
            trace("without play(null)");

            var stream:NetStream = new NetStream(connection);
            var bytes:ByteArray = new ByteArray();
            bytes.writeByte(1);

            try
            {
                trace("result=" + stream.appendBytes(bytes));
            }
            catch (e:Error)
            {
                trace(e);
            }
        }

        private function testAfterPlayUrl(connection:NetConnection):void
        {
            trace("after play(url)");

            var stream:NetStream = new NetStream(connection);
            stream.play("file.flv");

            var bytes:ByteArray = new ByteArray();
            bytes.writeByte(1);

            try
            {
                trace("result=" + stream.appendBytes(bytes));
            }
            catch (e:Error)
            {
                trace(e);
            }
        }

        private function testNull(connection:NetConnection):void
        {
            trace("appendBytes(null)");

            var stream:NetStream = new NetStream(connection);
            stream.play(null);

            try
            {
                trace("result=" + stream.appendBytes(null));
            }
            catch (e:Error)
            {
                trace(e);
            }
        }

        private function testReentrantAppend(connection:NetConnection):void
        {
            trace("reentrant append");

            var loader:URLLoader = new URLLoader();
            loader.dataFormat = URLLoaderDataFormat.BINARY;

            loader.addEventListener(Event.COMPLETE, function(event:Event):void
                {
                    var bytes:ByteArray = loader.data as ByteArray;

                    trace("FLV length=" + bytes.length);

                    var stream:NetStream = new NetStream(connection);

                    stream.client = {
                            onMetaData: function(info:Object):void
                            {
                                trace("onMetaData");

                                var extra:ByteArray = new ByteArray();

                                trace("before callback append");
                                stream.appendBytes(extra);
                                trace("after callback append");
                            }
                        };

                    stream.play(null);
                    stream.appendBytes(bytes);
                });

            loader.load(new URLRequest("result.flv"));
        }

    }
}
