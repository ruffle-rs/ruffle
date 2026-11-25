// The frame events (`enterFrame`, `frameConstructed`, `exitFrame` and
// `render`) are broadcast only to display objects. An EventDispatcher that
// isn't a display object never receives them, unless they are dispatched to
// it explicitly. Display objects receive the first three even when they're
// not on the display list. The test stops listening after frame 3.
package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.EventDispatcher;

    public class Test extends Sprite {
        private static const FRAME_EVENTS:Array = [
            Event.ENTER_FRAME,
            Event.FRAME_CONSTRUCTED,
            Event.EXIT_FRAME
        ];

        private var dispatcher:EventDispatcher = new EventDispatcher();
        private var sub:SubDispatcher = new SubDispatcher();
        private var offList:Sprite = new Sprite();
        private var frame:int = 0;
        private var listeners:Array = [];

        public function Test() {
            listen(dispatcher, "EventDispatcher", FRAME_EVENTS.concat(Event.RENDER));
            listen(sub, "SubDispatcher", FRAME_EVENTS.concat(Event.RENDER));
            listen(offList, "Sprite off the display list", FRAME_EVENTS);

            addEventListener(Event.ENTER_FRAME, onEnterFrame);
            addListener(this, Event.RENDER, function(e:Event):void {
                trace("Test: render on frame " + frame);
            });
        }

        private function addListener(target:EventDispatcher, type:String, listener:Function):void {
            target.addEventListener(type, listener);
            listeners.push([target, type, listener]);
        }

        private function listen(target:EventDispatcher, name:String, events:Array):void {
            for each (var type:String in events) {
                addListener(target, type, function(e:Event):void {
                    trace(name + ": " + e.type + " on frame " + frame);
                });
            }
        }

        private function onEnterFrame(e:Event):void {
            frame++;
            trace("Test: enterFrame on frame " + frame);

            // `render` is only dispatched after `invalidate`.
            stage.invalidate();

            if (frame == 4) {
                for each (var entry:Array in listeners) {
                    entry[0].removeEventListener(entry[1], entry[2]);
                }

                removeEventListener(Event.ENTER_FRAME, onEnterFrame);
                trace("Done");

                return;
            }

            if (frame == 2) {
                trace("Dispatching enterFrame explicitly");
                dispatcher.dispatchEvent(new Event(Event.ENTER_FRAME));
                sub.dispatchEvent(new Event(Event.ENTER_FRAME));
            }
        }
    }
}

import flash.events.EventDispatcher;

class SubDispatcher extends EventDispatcher {}
