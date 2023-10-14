package {
import flash.display.Sprite;

public class Test extends Sprite {}
}

import flash.events.EventDispatcher;
import flash.events.Event;

var event:CustomEvent = new CustomEvent("custom", false, false);


// Recursive redispatch test.
trace("recursive redispatch test");
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

// Non-recursive redispatch test.
trace("non-recursive redispatch test");
var dispatcher3:EventDispatcher = new EventDispatcher();
var dispatcher4:EventDispatcher = new EventDispatcher();

dispatcher3.addEventListener("custom", function(evt:CustomEvent) {
    trace("handler 1");
});

dispatcher4.addEventListener("custom", function(evt:CustomEvent) {
    trace("handler 2");
});

var event: CustomEvent = new CustomEvent("custom", false, false);

dispatcher3.dispatchEvent(event);
dispatcher4.dispatchEvent(event);