// compiled with mxmlc

import flash.display.MovieClip;
import flash.events.Event;
import flash.events.EventDispatcher;
import flash.events.IEventDispatcher;

package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.EventDispatcher;

    public class Test extends MovieClip {
        public function Test() {
        	var mc = new MovieClip();
        	addChild(mc);
        	var child = new MovieClip();
        	mc.addChild(child);

        	child.addEventListener("asdf", function(e){
        		trace("in child " + e.eventPhase);
        	});
        	mc.addEventListener("asdf", function(e){
        		trace("in mc " + e.eventPhase);
        	});
        	addEventListener("asdf", function(e){
        		trace("in root " + e.eventPhase);
        	});

        	child.dispatchEvent(new Event("asdf", true));

        	trace();
        	var dispatcher = new EventDispatcher(child);
        	dispatcher.addEventListener("asdf", function(e){
        		trace("in child's dispatcher " + e.eventPhase);
        	});

        	dispatcher.dispatchEvent(new Event("asdf", true));

        }

    }
}

class MyObject implements IEventDispatcher {
    private var dispatcher:EventDispatcher;
    public function MyObject() {
        dispatcher = new EventDispatcher(this);
    }
    public function addEventListener(type:String, listener:Function, useCapture:Boolean = false, priority:int = 0, useWeakReference:Boolean = false):void{
        dispatcher.addEventListener(type, listener, useCapture, priority);
    }
    public function dispatchEvent(evt:Event):Boolean{
        return dispatcher.dispatchEvent(evt);
    }
    public function hasEventListener(type:String):Boolean{
        return dispatcher.hasEventListener(type);
    }
    public function removeEventListener(type:String, listener:Function, useCapture:Boolean = false):void{
        dispatcher.removeEventListener(type, listener, useCapture);
    }
    public function willTrigger(type:String):Boolean {
        return dispatcher.willTrigger(type);
    }
}

var object = new MyObject();
object.dispatchEvent(new Event("asdf"));
object.addEventListener("asdf", function(){ trace("in listener") })
object.dispatchEvent(new Event("asdf"));
trace();
