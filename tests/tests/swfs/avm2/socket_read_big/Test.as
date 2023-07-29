package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.net.Socket;
import flash.net.ObjectEncoding;
import flash.events.Event;
import flash.events.ProgressEvent;
import flash.utils.ByteArray;
import flash.utils.Endian;

var socket:Socket = new Socket();

socket.endian = Endian.BIG_ENDIAN;

socket.addEventListener(Event.CONNECT, function(evt:Event):void {
    trace("connected");
});

socket.addEventListener(ProgressEvent.SOCKET_DATA, function(evt:ProgressEvent):void
{
    trace("data received");
    trace(evt);
    trace("Bytes available:");
    trace(socket.bytesAvailable);
    trace("--------")

    trace("readBoolean()");
    trace(socket.readBoolean());

    trace("");
    trace("readByte()");
    trace(socket.readByte());

    trace("");
    trace("readUnsignedByte()");
    trace(socket.readUnsignedByte());

    trace("");
    trace("readBytes()");
    var byteArray:ByteArray = new ByteArray();
    socket.readBytes(byteArray, 0, 3);
    trace(byteArray.readByte());
    trace(byteArray.readByte());
    trace(byteArray.readByte());

    trace("");
    trace("readDouble()");
    trace(socket.readDouble());

    trace("");
    trace("readFloat()");
    trace(socket.readFloat());

    trace("");
    trace("readInt()");
    trace(socket.readInt());

    trace("");
    trace("readMultiByte()");
    trace(socket.readMultiByte(6, "utf-8"));

    trace("");
    trace("readUnsignedShort()");
    trace(socket.readUnsignedShort());

    trace("");
    trace("readShort()");
    trace(socket.readShort());

    trace("");
    trace("readUnsignedInt()");
    trace(socket.readUnsignedInt());

    trace("");
    trace("readUTF()");
    trace(socket.readUTF());

    trace("");
    trace("readUTFBytes()");
    trace(socket.readUTFBytes(15));

    trace("");
    trace("close()");
    socket.close();
});

socket.connect("localhost", 8001);
