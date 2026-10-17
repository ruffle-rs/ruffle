package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var cmp:Function = function(x:int, y:int):Number {
            return x - y;
        };
        var noisy:Object = {
            valueOf: function():* { trace("valueOf called"); return 16; },
            toString: function():String { trace("toString called"); return "16"; }
        };

        run("()", []);
        run("(16)", [16]);
        run("(16.7)", [16.7]);
        run("(-1)", [-1]);
        run("(NaN)", [NaN]);
        run("(Infinity)", [Infinity]);
        run("(-Infinity)", [-Infinity]);
        run("(4294967312)", [4294967312]);
        run("(1e10)", [1e10]);
        run("(-16)", [-16]);
        run("(272)", [272]);
        run("(256)", [256]);
        run("(\"16\")", ["16"]);
        run("(\"abc\")", ["abc"]);
        run("(true)", [true]);
        run("(null)", [null]);
        run("(undefined)", [undefined]);
        run("({})", [{}]);
        run("([16])", [[16]]);
        run("(noisy)", [noisy]);
        run("(Number)", [Number]);
        run("(cmp)", [cmp]);
        run("(cmp, 2)", [cmp, 2]);

        for (var options:uint = 0; options < 32; options++) {
            trace("/// sort(" + options + ")");
            sortWithOptions(Vector.<int>([3, 20, 1, 10]), options);
            sortWithOptions(Vector.<String>(["b", "A", "c", "B"]), options);
            sortWithOptions(Vector.<int>([3, 1, 3]), options);
        }
    }

    private static function sortWithOptions(v:*, options:uint):void {
        var input:String = v.join(", ");
        try {
            var result:* = v.sort(options);
            var resultString:String = result === v ? "same vector" : String(result);
            trace("[" + input + "] returned " + resultString + ", vector: [" + v.join(", ") + "]");
        } catch (e:*) {
            trace("[" + input + "] caught: " + Object(e).constructor);
            trace(e.getStackTrace());
            trace("vector: [" + v.join(", ") + "]");
        }
    }

    private static function run(name:String, args:Array):void {
        trace("/// sort" + name);
        var v:Vector.<int> = new Vector.<int>();
        v.push(3, 20, 1, 10);
        try {
            var result:*;
            var f:Function = v.sort;
            result = f.apply(v, args);
            if (result === v) {
                trace("returned same vector");
            } else {
                trace("returned: " + result);
            }
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(String(e.getStackTrace()).replace(/@[0-9A-Fa-f]+/g, "@ADDRESS"));
        }
        trace("vector: [" + v.join(", ") + "]");
    }
}
}
