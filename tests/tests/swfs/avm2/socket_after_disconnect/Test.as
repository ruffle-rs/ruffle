package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.net.Socket;
import flash.events.Event;

var socket:Socket = new Socket();

socket.addEventListener(Event.CLOSE, function():void
{
    trace(socket.connected);
});

socket.connect("localhost", 8001);
