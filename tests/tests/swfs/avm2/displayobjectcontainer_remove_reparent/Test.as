package {
import flash.display.DisplayObject;
import flash.display.Sprite;
import flash.events.Event;

public class Test extends Sprite {
    private var a:Sprite;
    private var b:Sprite;
    private var other:Sprite;
    private var third:Sprite;
    private var logging:Boolean;

    public function Test() {
        var types:Array = [Event.REMOVED, Event.REMOVED_FROM_STAGE];
        for each (var t:String in types) {
            probe("removeChild, reparent to other", t, true, remover, adder("other"));
            probe("removeChildAt, reparent to other", t, true, function():* {
                return a.removeChildAt(1);
            }, adder("other"));
            probe("removeChild, reparent to other off stage", t, false, remover, adder("other"));
            probe("removeChild, move within parent", t, true, remover, function():void {
                a.addChildAt(b, 0);
            });
            probe("removeChild, reparent to other, then to third", t, true, remover,
                function():void {
                    other.addChild(b);
                    third.addChild(b);
                });
            probe("removeChildren, reparent to other", t, true, function():* {
                return a.removeChildren();
            }, adder("other"));
            probe("addChild, reparent to third", t, true, adder("other"), adder("third"));
            probe("addChild, move within new parent", t, true, adder("other"), function():void {
                other.addChildAt(b, 0);
            });
            probe("addChild, remove", t, true, adder("other"), remover);
            probe("addChild, remove and add back", t, true, adder("other"), function():void {
                remover();
                a.addChild(b);
            });
        }
        trace("// done");
    }

    private function remover():* {
        return a.removeChild(b);
    }

    private function adder(parent:String):Function {
        return function():* {
            return (parent == "other" ? other : third).addChild(b);
        };
    }

    // Stage > this > [a, third, other?], a > [c, b, d], other > [o]
    private function probe(name:String, type:String, otherOnStage:Boolean, trigger:Function,
            action:Function):void {
        trace("// " + name + ", in " + type);
        a = sprite("a", ["c", "b", "d"]);
        b = Sprite(a.getChildByName("b"));
        other = sprite("other", ["o"]);
        third = sprite("third", []);
        addChild(a);
        addChild(third);
        if (otherOnStage) {
            addChild(other);
        }

        var calls:int = 0;
        for each (var t:String in [Event.ADDED, Event.ADDED_TO_STAGE, Event.REMOVED,
                Event.REMOVED_FROM_STAGE]) {
            b.addEventListener(t, function(e:Event):void {
                if (!logging || e.target != b) {
                    return;
                }
                trace("  " + e.type + ": " + state());
                if (e.type == type && calls++ == 0) {
                    action();
                    trace("  listener done: " + state());
                }
            });
        }

        logging = true;
        var result:* = trigger();
        trace("  returned " + (result == b ? "b" : result) + ": " + state());
        logging = false;
        while (numChildren > 0) {
            removeChildAt(0);
        }
    }

    private function sprite(name:String, children:Array):Sprite {
        var s:Sprite = new Sprite();
        s.name = name;
        for each (var child:String in children) {
            s.addChild(sprite(child, []));
        }
        return s;
    }

    private function state():String {
        return "parent " + (b.parent ? b.parent.name : null) + ", on stage " + (b.stage != null)
            + ", a " + names(a) + ", other " + names(other) + ", third " + names(third);
    }

    private function names(s:Sprite):String {
        var result:Array = [];
        for (var i:int = 0; i < s.numChildren; i++) {
            result.push(s.getChildAt(i).name);
        }
        return "[" + result.join(",") + "]";
    }
}
}
