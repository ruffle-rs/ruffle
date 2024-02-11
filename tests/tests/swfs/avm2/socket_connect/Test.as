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
    trace("Event handler: socket.connected = " + socket.connected);
    socket.writeUTF("Hello!");
    socket.flush();
    socket.close();
    trace("After close: socket.connected = " + socket.connected);
});

trace("Before call: socket.connected = " + socket.connected);
socket.connect("localhost", 8001);
trace("After call: socket.connected = " + socket.connected);

