package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.net.XMLSocket;
import flash.errors.IOError;
import flash.events.Event;
import flash.events.DataEvent;

// NOTE: prettyPrinting affects XMLSocket, so turn it off so we can pass.
XML.prettyPrinting = false;

var socket:XMLSocket = new XMLSocket();

trace("Timeout clamp");
trace(socket.timeout);
socket.timeout = 0;
trace(socket.timeout);

trace("Socket errors");
try
{
    socket.close();
}
catch (e:IOError)
{
    trace(e);
}

try
{
    socket.send("Hello!");
}
catch (e:IOError)
{
    trace(e);
}

socket.addEventListener(Event.CONNECT, function(evt:Event):void
{
    trace("connected");
    socket.send("Hello!");

    var xml:XML = <root><item>Hello world!</item></root>;

    socket.send(xml);

    var onlyRoot:XML = <root>Hello!</root>;
    socket.send(onlyRoot);
});

socket.addEventListener(DataEvent.DATA, function(evt:DataEvent):void
{
    trace(evt);
});

socket.addEventListener(Event.CLOSE, function(evt:Event):void
{
    trace("closed");
});

socket.connect("localhost", 8001);
