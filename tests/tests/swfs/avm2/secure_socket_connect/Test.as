package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.events.Event;
import flash.events.IOErrorEvent;
import flash.events.SecurityErrorEvent;
import flash.net.SecureSocket;

trace("SecureSocket.isSupported = " + SecureSocket.isSupported);

var socket:SecureSocket = new SecureSocket();
trace("Initial serverCertificateStatus = " + socket.serverCertificateStatus);
trace("Initial connected = " + socket.connected);

socket.addEventListener(Event.CONNECT, function(event:Event):void
{
    trace("Event: connect");
    trace("connected = " + socket.connected);
    trace("serverCertificateStatus = " + socket.serverCertificateStatus);
    socket.writeUTF("Hello Secure!");
    socket.flush();
    socket.close();
    trace("After close: connected = " + socket.connected);
});

socket.addEventListener(IOErrorEvent.IO_ERROR, function(event:IOErrorEvent):void
{
    trace("Event: ioError - " + event.text);
});

socket.addEventListener(SecurityErrorEvent.SECURITY_ERROR, function(event:SecurityErrorEvent):void
{
    trace("Event: securityError - " + event.text);
});

socket.connect("localhost", 8001);
trace("After connect call: connected = " + socket.connected);
