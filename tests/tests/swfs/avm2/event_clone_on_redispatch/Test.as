package {
import flash.display.Sprite;

public class Test extends Sprite {}
}

import flash.events.EventDispatcher;
import flash.events.Event;

var event:CustomEvent = new CustomEvent("custom", false, false);

var dispatcher1:EventDispatcher = new EventDispatcher();
var dispatcher2:EventDispatcher = new EventDispatcher();

dispatcher1.addEventListener("custom", function(event:CustomEvent):void {
    trace("first handler called");
    dispatcher2.dispatchEvent(event);
});

dispatcher2.addEventListener("custom", function(event:CustomEvent):void {
    trace("second handler called");
});

dispatcher1.dispatchEvent(event);