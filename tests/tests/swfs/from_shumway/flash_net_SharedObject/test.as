/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf SharedObject,100,100,2 test/swfs/flash_net_SharedObject.as

   This template is for writing SWFs using pure AS3. It allows for testing UI events, screen shots
   and trace log using the Shumway test harness.


*/

package {

    import flash.display.Sprite;
    import flash.events.Event;

    public class SharedObjectTest extends Sprite {
        public var loader;
        public function SharedObjectTest() {
            var child = new TestObject();
            addChild(child);
            addEventListener(Event.ENTER_FRAME, child.enterFrameHandler);
        }
    }
}

import flash.display.*;
import flash.events.*;
import flash.net.*;

class TestObject extends Sprite {
    private var bgColor: uint = 0xFFCC00;
    private var pos: uint     = 10;
    private var size: uint    = 80;

    /*
      Install event listeners for testing events, and construct and add child
      objects.
    */

    var sharedObject: SharedObject;

    public function TestObject() {
        SharedObject.defaultObjectEncoding = ObjectEncoding.AMF0;
        this.sharedObject = SharedObject.getLocal("ShareObjectTest");
    }

    private var frameCount = 0;

    /*
      Set and get properties that have a UI affect to test both screen
      capture and property values.
    */

    function enterFrameHandler(event:Event):void {
        frameCount++;
        var target = event.target;
        var loader = target.loader;
        switch (frameCount) {
        case 1:
            (function () {
                sharedObject.data.x = 10;
                sharedObject.flush();
                trace('stored value of `x`: ' + sharedObject.data.x);
                trace('stored data\'s size: ' + sharedObject.size);
                sharedObject.clear();
                trace('stored value of `x` after clear: ' + sharedObject.data.x);
                trace('stored data\'s size after clear: ' + sharedObject.size);
            })();
            break;
        case 2:
            (function () {
            })();
            break;
        case 3:
            (function () {
            })();
            break;
        default:
            parent.removeEventListener(Event.ENTER_FRAME, enterFrameHandler);
            break;
        }
    }
}
