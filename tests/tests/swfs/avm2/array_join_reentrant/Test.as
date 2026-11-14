package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        run("push", function(a:Array):void { a.push("pushed"); });
        run("pop", function(a:Array):void { a.pop(); });
        run("shift", function(a:Array):void { a.shift(); });
        run("unshift", function(a:Array):void { a.unshift("unshifted"); });
        run("set next element", function(a:Array):void { a[2] = "changed"; });
        run("set previous element", function(a:Array):void { a[0] = "changed"; });
        run("set far element", function(a:Array):void { a[6] = "far"; });
        run("length = 0", function(a:Array):void { a.length = 0; });
        run("length = 10", function(a:Array):void { a.length = 10; });
        run("splice", function(a:Array):void { a.splice(0, 2, "s1", "s2", "s3"); });
        run("reverse", function(a:Array):void { a.reverse(); });
        run("join", function(a:Array):void { trace("nested join: " + a.join("+")); });

        runToString();
        runSeparator();
        runHole();
    }

    private static function makeArray(action:Function):Array {
        var a:Array = [];
        a.push("first");
        a.push(new Noisy("noisy", function():void { action(a); }));
        a.push("third");
        a.push("fourth");
        return a;
    }

    private static function run(name:String, action:Function):void {
        trace("/// " + name);
        var a:Array = makeArray(action);
        try {
            trace("result: " + a.join("-"));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(a);
    }

    private static function runToString():void {
        trace("/// toString, push");
        var a:Array = makeArray(function(a:Array):void { a.push("pushed"); });
        try {
            trace("result: " + a.toString());
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(a);
    }

    private static function runSeparator():void {
        trace("/// separator, push");
        var a:Array = ["a", "b", "c"];
        var sep:Noisy = new Noisy("|", function():void { a.push("pushed"); });
        try {
            trace("result: " + a.join(sep));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(a);

        trace("/// separator, length = 0");
        a = ["a", "b", "c"];
        sep = new Noisy("|", function():void { a.length = 0; });
        try {
            trace("result: " + a.join(sep));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(a);
    }

    private static function runHole():void {
        trace("/// hole from prototype, push");
        var a:Array = ["a"];
        a[2] = "c";
        Array.prototype[1] = new Noisy("proto", function():void { a.push("pushed"); });
        try {
            trace("result: " + a.join("-"));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        delete Array.prototype[1];
        dump(a);
    }

    private static function dump(a:Array):void {
        var out:Array = [];
        for (var i:int = 0; i < a.length; i++) {
            var v:* = a[i];
            out.push(v is Noisy ? Noisy(v).name : String(v));
        }
        trace("length: " + a.length + ", elements: [" + out.join(", ") + "]");
    }
}
}

class Noisy {
    public var name:String;
    private var action:Function;
    private var calls:int = 0;

    public function Noisy(name:String, action:Function) {
        this.name = name;
        this.action = action;
    }

    public function toString():String {
        calls++;
        trace("toString called on " + name + " (" + calls + ")");
        if (calls == 1) {
            action();
        }
        return name;
    }
}
