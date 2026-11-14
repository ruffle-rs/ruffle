package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        run("push", false, function(v:Vector.<Object>):void { v.push("pushed"); });
        run("pop", false, function(v:Vector.<Object>):void { v.pop(); });
        run("shift", false, function(v:Vector.<Object>):void { v.shift(); });
        run("unshift", false, function(v:Vector.<Object>):void { v.unshift("unshifted"); });
        run("set next element", false, function(v:Vector.<Object>):void { v[2] = "changed"; });
        run("set previous element", false, function(v:Vector.<Object>):void { v[0] = "changed"; });
        run("length = 0", false, function(v:Vector.<Object>):void { v.length = 0; });
        run("length = 10", false, function(v:Vector.<Object>):void { v.length = 10; });
        run("splice", false, function(v:Vector.<Object>):void { v.splice(0, 2, "s1", "s2", "s3"); });
        run("reverse", false, function(v:Vector.<Object>):void { v.reverse(); });
        run("sort", false, function(v:Vector.<Object>):void { v.sort(0); });
        run("join", false, function(v:Vector.<Object>):void { trace("nested join: " + v.join("+")); });
        run("fixed, push", true, function(v:Vector.<Object>):void { v.push("pushed"); });
        run("fixed, set next element", true, function(v:Vector.<Object>):void { v[2] = "changed"; });
        run("fixed, unfix and push", true, function(v:Vector.<Object>):void { v.fixed = false; v.push("pushed"); });

        runToString();
        runSeparator();
    }

    private static function makeVector(fixed:Boolean, action:Function):Vector.<Object> {
        var v:Vector.<Object> = new Vector.<Object>();
        v.push("first");
        v.push(new Noisy("noisy", function():void { action(v); }));
        v.push("third");
        v.push("fourth");
        v.fixed = fixed;
        return v;
    }

    private static function run(name:String, fixed:Boolean, action:Function):void {
        trace("/// " + name);
        var v:Vector.<Object> = makeVector(fixed, action);
        try {
            trace("result: " + v.join("-"));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(v);
    }

    private static function runToString():void {
        trace("/// toString, push");
        var v:Vector.<Object> = makeVector(false, function(v:Vector.<Object>):void { v.push("pushed"); });
        try {
            trace("result: " + v.toString());
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(v);
    }

    private static function runSeparator():void {
        trace("/// separator, push");
        var v:Vector.<Object> = new Vector.<Object>();
        v.push("a", "b", "c");
        var sep:Noisy = new Noisy("|", function():void { v.push("pushed"); });
        try {
            trace("result: " + v.join(sep));
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
        dump(v);
    }

    private static function dump(v:Vector.<Object>):void {
        var out:Array = [];
        for (var i:int = 0; i < v.length; i++) {
            var e:* = v[i];
            out.push(e is Noisy ? Noisy(e).name : String(e));
        }
        trace("length: " + v.length + ", fixed: " + v.fixed + ", elements: [" + out.join(", ") + "]");
    }
}
}
