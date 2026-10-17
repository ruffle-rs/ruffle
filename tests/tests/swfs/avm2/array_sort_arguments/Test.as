package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var cmp:Function = function(x:*, y:*):Number {
            return Number(x) - Number(y);
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
        run("(cmp, 2.5)", [cmp, 2.5]);
        run("(cmp, 4294967298)", [cmp, 4294967298]);
        run("(cmp, Infinity)", [cmp, Infinity]);
        run("(cmp, -1)", [cmp, -1]);
        run("(cmp, \"2\")", [cmp, "2"]);
        run("(cmp, true)", [cmp, true]);
        run("(cmp, null)", [cmp, null]);
        run("(cmp, undefined)", [cmp, undefined]);
        run("(cmp, {})", [cmp, {}]);
        run("(cmp, noisy)", [cmp, noisy]);
        run("(cmp, 2, \"extra\")", [cmp, 2, "extra"]);
        run("(null, 16)", [null, 16]);
        run("(undefined, 16)", [undefined, 16]);
        run("({}, 16)", [{}, 16]);
        run("(16, 2)", [16, 2]);
        run("(16, {})", [16, {}]);
        run("(\"16\", 2)", ["16", 2]);
        run("(16, 2, \"extra\")", [16, 2, "extra"]);

        for (var options:uint = 0; options < 32; options++) {
            trace("/// sort(" + options + ")");
            sortWithOptions([3, 20, 1, 10], options);
            sortWithOptions(["b", "A", "c", "B"], options);
            sortWithOptions([3, 1, 3], options);
        }
    }

    private static function sortWithOptions(a:Array, options:uint):void {
        var input:String = a.join(", ");
        try {
            var result:* = a.sort(options);
            var resultString:String;
            if (result === a) {
                resultString = "same array";
            } else if (result is Array) {
                resultString = "[" + result.join(", ") + "]";
            } else {
                resultString = String(result);
            }
            trace("[" + input + "] returned " + resultString + ", array: [" + a.join(", ") + "]");
        } catch (e:*) {
            trace("[" + input + "] caught: " + Object(e).constructor);
            trace(e.getStackTrace());
            trace("array: [" + a.join(", ") + "]");
        }
    }

    private static function run(name:String, args:Array):void {
        trace("/// sort" + name);
        var a:Array = [3, 20, 1, 10];
        try {
            var result:*;
            switch (args.length) {
                case 0: result = a.sort(); break;
                case 1: result = a.sort(args[0]); break;
                case 2: result = a.sort(args[0], args[1]); break;
                default: result = a.sort(args[0], args[1], args[2]); break;
            }
            if (result === a) {
                trace("returned same array");
            } else if (result is Array) {
                trace("returned: [" + result.join(", ") + "]");
            } else {
                trace("returned: " + result);
            }
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(String(e.getStackTrace()).replace(/@[0-9A-Fa-f]+/g, "@ADDRESS"));
        }
        trace("array: [" + a.join(", ") + "]");
    }
}
}
