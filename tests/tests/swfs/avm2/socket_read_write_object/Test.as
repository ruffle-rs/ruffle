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

var socket:Socket = new Socket();

socket.addEventListener(Event.CONNECT, function(evt:Event):void
{
    trace("Connected");
    trace("writeObject() AMF3")
    socket.objectEncoding = ObjectEncoding.AMF3;
    socket.writeObject(new Object());
    trace("writeObject() AMF0")
    socket.objectEncoding = ObjectEncoding.AMF0;
    socket.writeObject(new Object());
    socket.flush();
});

socket.addEventListener(ProgressEvent.SOCKET_DATA, function(evt:ProgressEvent):void
{
    trace(evt);
    trace("readObject() AMF3")
    socket.objectEncoding = ObjectEncoding.AMF3;
    trace(socket.readObject());
    trace("readObject() AMF0")
    socket.objectEncoding = ObjectEncoding.AMF0;
    trace(socket.readObject());
    socket.close();
});

socket.connect("localhost", 8001);
