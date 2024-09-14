/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf URLRequestTest,100,100 test/swfs/flash_net_URLRequest.as
*/

package {

    import flash.display.Sprite;
    import flash.events.Event;

    public class URLRequestTest extends Sprite {
        public var loader;
        public function URLRequestTest() {
            loader = new TestObject();
        }
    }
}

import flash.display.*;
import flash.events.*;
import flash.net.*;

class TestObject extends URLLoader {
    private var bgColor: uint = 0xFFCC00;
    private var pos: uint     = 10;
    private var size: uint    = 80;
    private var url           = "test.as";

    private var request;

    public function TestObject() {
        configureListeners(this);
        request = new URLRequest();
        request.url = "data.txt";
        request.data = { "foo": 10, "bar": 20 };
        request.method = "GET";
        load(request);
    }

    private function configureListeners(dispatcher:IEventDispatcher):void {
        dispatcher.addEventListener(Event.COMPLETE, completeHandler);
    }

    private function completeHandler(event:Event):void {
        var data = event.target.data;
        var result = data.split("\n")[0] === "BOO!" ? "PASS" : "FAIL";
        trace(result + ": flash.net::URLRequest/set url ()");
        trace(result + ": flash.net::URLRequest/set data ()");
        trace(result + ": flash.net::URLRequest/set method ()");
        var result = request.url === "data.txt" ? "PASS" : "FAIL";
        trace(result + ": flash.net::URLRequest/get url ()");
        var result = request.data.bar === 20 ? "PASS" : "FAIL";
        trace(result + ": flash.net::URLRequest/get data ()");
        var result = request.method === "GET" ? "PASS" : "FAIL";
        trace(result + ": flash.net::URLRequest/get method ()");
    }
}
