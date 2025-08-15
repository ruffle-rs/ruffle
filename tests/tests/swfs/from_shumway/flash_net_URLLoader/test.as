/* -*- Mode: java; indent-tabs-mode: nil -*- */
/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf URLLoaderTest,100,100,2 test/swfs/flash_net_URLLoader.as
*/

package {

    import flash.display.Sprite;
    import flash.events.Event;

    public class URLLoaderTest extends Sprite {
        public var loader;
        public function URLLoaderTest() {
            loader = new CustomURLLoader();
        }
    }
}

import flash.display.*;
import flash.events.*;
import flash.net.*;

class CustomURLLoader extends URLLoader {
    private var bgColor: uint = 0xFFCC00;
    private var pos: uint     = 10;
    private var size: uint    = 80;
    private var url           = "test.as";

    public function CustomURLLoader() {
        configureListeners(this);
        var request:URLRequest = new URLRequest(url);
        load(request);
    }

    private function configureListeners(dispatcher:IEventDispatcher):void {
        dispatcher.addEventListener(Event.COMPLETE, completeHandler);
        dispatcher.addEventListener(HTTPStatusEvent.HTTP_STATUS, httpStatusHandler);
        dispatcher.addEventListener(Event.OPEN, openHandler);
        dispatcher.addEventListener(ProgressEvent.PROGRESS, progressHandler);
        dispatcher.addEventListener(Event.UNLOAD, unloadHandler);
    }

    var ticket = 1;
    var completeHandlerTicket = 0;
    var httpStatusHandlerTicket = 0;
    var openHandlerTicket = 0;
    var progressHandlerTicket = 0;
    var unloadHandlerTicket = 0;

    private function completeHandler(event:Event):void {
        trace("completeHandler: " + ticket);
        completeHandlerTicket = ticket++;
        var data = event.target.data;
        var lines = data.split("\n");
        trace(data.length);
        var result = lines[0] === "/* -*- Mode: java; indent-tabs-mode: nil -*- */" ? "PASS" : "FAIL";
        trace(result + ": flash.net::URLLoader/load ()");
        trace("PASS" + ": flash.net::URLLoader/addEventListener ()");
    }
    
    private function httpStatusHandler(event:HTTPStatusEvent):void {
        trace("httpStatusHandler: " + ticket);
        httpStatusHandlerTicket = ticket;
    }

    private function openHandler(event:Event):void {
        trace("openHandler: " + ticket);
        openHandlerTicket = ticket++;
    }

    private function progressHandler(event:ProgressEvent):void {
        trace("progressHandler: " + ticket + " bytesLoaded=" + event.bytesLoaded + " bytesTotal=" + event.bytesTotal);
        progressHandlerTicket = ticket;
    }

    private function unloadHandler(event:Event):void {
        trace("unloadHandler: " + ticket);
        unloadHandlerTicket = ticket;
    }
}
