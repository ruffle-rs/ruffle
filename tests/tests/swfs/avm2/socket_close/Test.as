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

socket.addEventListener(Event.CONNECT, function(evt:Event):void
{
    trace("connected");
});

socket.addEventListener(Event.CLOSE, function(evt:Event):void
{
    trace("closed");
});

socket.connect("localhost", 8001);
