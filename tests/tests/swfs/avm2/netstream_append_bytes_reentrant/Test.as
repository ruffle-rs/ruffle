package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.NetStatusEvent;
    import flash.net.NetConnection;
    import flash.net.NetStream;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        private var connection:NetConnection = new NetConnection();
        private var streams:Array = [];
        private var appendedEvents:Array = [];
        private var replacedEvents:Array = [];
        private var directEvents:Array = [];
        private var appended:Boolean = false;
        private var replaced:Boolean = false;
        private var frames:int = 0;

        public function Test() {
            connection.connect(null);

            testAppend();
            testReplace();
            testDirect();

            addEventListener(Event.ENTER_FRAME, finish);
        }

        private function write24(bytes:ByteArray, value:uint):void {
            bytes.writeByte(value >> 16);
            bytes.writeByte(value >> 8);
            bytes.writeByte(value);
        }

        private function metadata(marker:String, header:Boolean):ByteArray {
            var body:ByteArray = new ByteArray();
            body.writeByte(2);
            body.writeUTF("onMetaData");
            body.writeByte(8);
            body.writeUnsignedInt(1);
            body.writeUTF("marker");
            body.writeByte(2);
            body.writeUTF(marker);
            body.writeShort(0);
            body.writeByte(9);

            var bytes:ByteArray = new ByteArray();

            if (header) {
                bytes.writeUTFBytes("FLV");
                bytes.writeByte(1);
                bytes.writeByte(0);
                bytes.writeUnsignedInt(9);
                bytes.writeUnsignedInt(0);
            }

            bytes.writeByte(18);
            write24(bytes, body.length);
            write24(bytes, 0);
            bytes.writeByte(0);
            write24(bytes, 0);
            bytes.writeBytes(body);
            bytes.writeUnsignedInt(11 + body.length);
            bytes.position = 0;
            return bytes;
        }

        private function testAppend():void {
            var stream:NetStream = new NetStream(connection);
            streams.push(stream);
            stream.addEventListener(NetStatusEvent.NET_STATUS, function(event:NetStatusEvent):void {
                if (event.info.level == "error") {
                    trace("stream error=" + event.info.code);
                }
            });

            stream.client = {
                onMetaData: function(info:Object):void {
                    appendedEvents.push(info.marker);

                    if (info.marker == "initial" && !appended) {
                        appended = true;
                        stream.appendBytes(metadata("appended", false));
                    }
                }
            };

            stream.play(null);
            stream.appendBytes(metadata("initial", true));
        }

        private function testReplace():void {
            var stream:NetStream = new NetStream(connection);
            streams.push(stream);
            stream.addEventListener(NetStatusEvent.NET_STATUS, function(event:NetStatusEvent):void {
                if (event.info.level == "error") {
                    trace("stream error=" + event.info.code);
                }
            });

            stream.client = {
                onMetaData: function(info:Object):void {
                    replacedEvents.push(info.marker === undefined ? "replacement" : info.marker);

                    if (info.marker == "initial" && !replaced) {
                        replaced = true;
                        stream.appendBytes(metadata("stale", false));
                        stream.play(loaderInfo.url.substring(
                            0, loaderInfo.url.lastIndexOf("/") + 1
                        ) + "replacement.flv");
                    }
                }
            };

            stream.play(null);
            stream.appendBytes(metadata("initial", true));
        }

        private function testDirect():void {
            var stream:NetStream = new NetStream(connection);
            streams.push(stream);

            stream.addEventListener(NetStatusEvent.NET_STATUS, function(event:NetStatusEvent):void {
                if (event.info.level == "error") {
                    directEvents.push(event.info.code);
                }
            });

            stream.client = {
                onMetaData: function(info:Object):void {
                    directEvents.push("metadata");
                }
            };

            stream.play(loaderInfo.url.substring(
                0, loaderInfo.url.lastIndexOf("/") + 1
            ) + "replacement.flv");
        }

        private function finish(event:Event):void {
            if (++frames < 60) {
                return;
            }

            removeEventListener(Event.ENTER_FRAME, finish);

            trace("reentrant callbacks=" + appendedEvents.join(","));
            trace("replacement callbacks=" + replacedEvents.join(","));
            trace("direct callbacks=" + directEvents.join(","));
            trace("APPEND_BYTES_REENTRANT_COMPLETE");

            for each (var stream:NetStream in streams) {
                stream.close();
            }
        }
    }
}
