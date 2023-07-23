package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.events.Event;
import flash.net.Socket;

var socket:Socket = new Socket();

socket.addEventListener(Event.CONNECT, function(event:Event):void
{
    trace("connected");
    socket.writeUTF("Hello!");
    socket.flush();
    socket.close();
});

socket.connect("localhost", 8001);

