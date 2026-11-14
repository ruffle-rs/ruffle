package {
import flash.display.DisplayObject;
import flash.display.Sprite;
import flash.events.Event;

public class Test extends Sprite {
    private var container:Sprite;
    private var logging:Boolean = true;

    public function Test() {
        trace("// removeChildren()");
        setup(["a", "b", "c"]);
        container.removeChildren();
        traceChildren();

        trace("// removeChildren(1, 2)");
        setup(["a", "b", "c", "d"]);
        container.removeChildren(1, 2);
        traceChildren();

        trace("// listener adds a child at the start");
        setup(["a", "b", "c", "d"]);
        var added:Sprite = new Sprite();
        added.name = "added";
        addOnce(container.getChildByName("a"), function():void {
            container.addChildAt(added, 0);
        });
        container.removeChildren();
        traceChildren();

        trace("// listener adds a child at the end");
        setup(["a", "b", "c"]);
        added = new Sprite();
        added.name = "added";
        addOnce(container.getChildByName("a"), function():void {
            container.addChild(added);
        });
        container.removeChildren();
        traceChildren();

        trace("// listener removes a later child");
        setup(["a", "b", "c", "d"]);
        var c:DisplayObject = container.getChildByName("c");
        addOnce(container.getChildByName("a"), function():void {
            container.removeChild(c);
        });
        container.removeChildren();
        traceChildren();

        trace("// listener moves a later child to another container");
        setup(["a", "b", "c", "d"]);
        var other:Sprite = new Sprite();
        addChild(other);
        c = container.getChildByName("c");
        addOnce(container.getChildByName("a"), function():void {
            other.addChild(c);
        });
        container.removeChildren();
        traceChildren();
        trace("c in other: " + (c.parent == other));
        removeChild(other);

        trace("// done");
    }

    private function setup(names:Array):void {
        if (container != null) {
            logging = false;
            removeChild(container);
            logging = true;
        }
        container = new Sprite();
        addChild(container);
        for each (var name:String in names) {
            var child:Sprite = new Sprite();
            child.name = name;
            child.addEventListener(Event.REMOVED, onEvent);
            child.addEventListener(Event.REMOVED_FROM_STAGE, onEvent);
            container.addChild(child);
        }
    }

    private function onEvent(e:Event):void {
        if (!logging || e.target != e.currentTarget) {
            return;
        }
        var target:DisplayObject = DisplayObject(e.target);
        trace(e.type + " " + target.name + ", in container: " + (target.parent == container)
            + ", children: " + childNames());
    }

    private function addOnce(child:DisplayObject, action:Function):void {
        var called:Boolean = false;
        child.addEventListener(Event.REMOVED, function(e:Event):void {
            if (!called) {
                called = true;
                action();
            }
        });
    }

    private function childNames():String {
        var names:Array = [];
        for (var i:int = 0; i < container.numChildren; i++) {
            names.push(container.getChildAt(i).name);
        }
        return "[" + names.join(",") + "]";
    }

    private function traceChildren():void {
        trace("children: " + childNames());
    }
}
}
