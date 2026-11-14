package {
import flash.display.DisplayObject;
import flash.display.Sprite;
import flash.events.Event;

public class Test extends Sprite {
    private var a:Sprite;
    private var b:Sprite;
    private var c:Sprite;
    private var d:Sprite;
    private var g:Sprite;
    private var other:Sprite;

    public function Test() {
        begin("removeChild in removed");
        probe(Event.REMOVED, remover(a, b), remover(a, b));

        begin("removeChild in removedFromStage");
        probe(Event.REMOVED_FROM_STAGE, remover(a, b), remover(a, b));

        begin("removeChildAt in removed");
        probe(Event.REMOVED, function():void {
            a.removeChildAt(1);
        }, function():void {
            a.removeChildAt(a.getChildIndex(b));
        });

        begin("removeChildren in removed");
        probe(Event.REMOVED, a.removeChildren, a.removeChildren);

        begin("removeChildren in removed of removeChild");
        probe(Event.REMOVED, remover(a, b), a.removeChildren);

        begin("removeChild in a nested event");
        probe(Event.REMOVED, remover(a, b), function():void {
            var dispatcher:Sprite = new Sprite();
            dispatcher.addEventListener("nested", function(e:Event):void {
                attempt(remover(a, b));
            });
            dispatcher.dispatchEvent(new Event("nested"));
        });

        begin("removeChild of a non-child in removed");
        probe(Event.REMOVED, remover(a, b), remover(other, b));

        begin("removeChild of a sibling in removed");
        probe(Event.REMOVED, remover(a, b), remover(a, c));

        begin("removeChild in removed of reparenting");
        probe(Event.REMOVED, function():void {
            other.addChild(b);
        }, remover(a, b));

        begin("removeChild after a re-entrant removal");
        once(b, Event.REMOVED, b, remover(a, b));
        a.removeChild(b);
        a.addChild(b);
        run(remover(a, b));

        begin("removeChild of self in removedFromStage of an ancestor");
        var calls:int = 0;
        b.addEventListener(Event.REMOVED_FROM_STAGE, function(e:Event):void {
            calls++;
            trace("  removedFromStage #" + calls);
            attempt(remover(a, b));
        });
        run(remover(this, a));

        begin("mutual removeChild in removed");
        once(b, Event.REMOVED, b, remover(a, c));
        once(c, Event.REMOVED, c, remover(a, b));
        run(remover(a, b));

        begin("mutual removeChild in removed and removedFromStage");
        once(b, Event.REMOVED, b, remover(a, c));
        once(c, Event.REMOVED_FROM_STAGE, c, remover(a, b));
        run(remover(a, b));

        begin("removeChild in removed on the parent");
        once(a, Event.REMOVED, b, remover(a, b));
        run(remover(a, b));

        begin("removeChild in removed captured on an ancestor");
        once(this, Event.REMOVED, b, remover(a, b), true);
        run(remover(a, b));

        begin("removeChild of the parent in removedFromStage of a grandchild");
        once(g, Event.REMOVED_FROM_STAGE, g, remover(a, b));
        run(remover(a, b));

        begin("removeChild of self in removedFromStage of a grandchild");
        once(g, Event.REMOVED_FROM_STAGE, g, remover(b, g));
        run(remover(a, b));

        begin("removeChild of the parent in removed");
        once(b, Event.REMOVED, b, remover(this, a));
        run(remover(a, b));

        trace("// done");
    }

    // Stage > this > [a, other], a > [c, b, d], b > g
    private function begin(name:String):void {
        trace("// " + name);
        a = sprite("a");
        b = sprite("b");
        c = sprite("c");
        d = sprite("d");
        g = sprite("g");
        other = sprite("other");
        addChild(a);
        addChild(other);
        a.addChild(c);
        a.addChild(b);
        a.addChild(d);
        b.addChild(g);
    }

    private function sprite(name:String):Sprite {
        var s:Sprite = new Sprite();
        s.name = name;
        return s;
    }

    private function probe(type:String, trigger:Function, action:Function):void {
        var calls:int = 0;
        b.addEventListener(type, function(e:Event):void {
            if (e.target != b) {
                return;
            }
            calls++;
            trace("  " + type + " #" + calls);
            if (calls == 1) {
                attempt(action);
            }
        });
        run(trigger);
    }

    private function once(on:DisplayObject, type:String, target:DisplayObject, action:Function,
            useCapture:Boolean = false):void {
        var called:Boolean = false;
        on.addEventListener(type, function(e:Event):void {
            if (called || e.target != target) {
                return;
            }
            called = true;
            trace("  " + type + " of " + nameOf(target) + " on " + nameOf(on));
            attempt(action);
        }, useCapture);
    }

    private function remover(parent:Sprite, child:DisplayObject):Function {
        return function():void {
            parent.removeChild(child);
        };
    }

    private function run(trigger:Function):void {
        attempt(trigger);
        traceState();
        while (numChildren > 0) {
            removeChildAt(0);
        }
    }

    private function attempt(f:Function):void {
        try {
            f();
            trace("  returned");
        } catch (e:*) {
            trace("  caught: " + Object(e).constructor);
            trace("  " + Object(e).getStackTrace());
        }
    }

    private function traceState():void {
        trace("  a: " + nameOf(a.parent) + ", b: " + nameOf(b.parent) + ", c: " + nameOf(c.parent)
            + ", d: " + nameOf(d.parent) + ", g: " + nameOf(g.parent));
    }

    private function nameOf(o:DisplayObject):String {
        if (o == null) return "null";
        if (o == this) return "this";
        return o.name;
    }
}
}
