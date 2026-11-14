package {
import flash.display.Sprite;

public class Test extends Sprite {
    public static var throwFromSetter:Boolean = false;

    public function Test() {
        var lengths:Array = [
            0, -0, 5, 4294967295,
            -1, 1.5, NaN, Infinity, -Infinity, 4294967296, 1e10
        ];
        for each (var length:Number in lengths) {
            run("new Array(" + length + ")", function():Array { return new Array(length); });
        }

        var intLength:int = -1;
        run("new Array(int -1)", function():Array { return new Array(intLength); });
        var uintLength:uint = 7;
        run("new Array(uint 7)", function():Array { return new Array(uintLength); });

        run("Array(5)", function():Array { return Array(5); });
        run("Array(-1)", function():Array { return Array(-1); });
        run("Array(1.5)", function():Array { return Array(1.5); });

        var noisy:Object = {
            valueOf: function():* { trace("valueOf called"); return 5; },
            toString: function():String { trace("toString called"); return "5"; }
        };
        var values:Array = [
            "5", "abc", "", true, false, null, undefined, noisy,
            new Number(5), new int(5), new uint(5), [5], new Date(0)
        ];
        for (var j:int = 0; j < values.length; j++) {
            var value:* = values[j];
            run("new Array(values[" + j + "]: " + typeof value + ")", function():Array { return new Array(value); });
        }
        var untypedInt:* = int(-3);
        run("new Array(* int -3)", function():Array { return new Array(untypedInt); });
        var untypedUint:* = uint(3);
        run("new Array(* uint 3)", function():Array { return new Array(untypedUint); });
        run("new Array(5, 6)", function():Array { return new Array(5, 6); });
        run("new Array(-1, 6)", function():Array { return new Array(-1, 6); });
        run("new Array(1.5, \"a\")", function():Array { return new Array(1.5, "a"); });

        run("new ArraySub0()", function():Array { return new ArraySub0(); });
        run("new ArraySub1(5)", function():Array { return new ArraySub1(5); });
        run("new ArraySub1(0)", function():Array { return new ArraySub1(0); });
        run("new ArraySub1(\"5\")", function():Array { return new ArraySub1("5"); });
        run("new ArraySub1(-1)", function():Array { return new ArraySub1(-1); });
        run("new ArraySub3(1, 2, 3)", function():Array { return new ArraySub3(1, 2, 3); });

        throwFromSetter = true;
        run("new ArraySub0(), setter throws", function():Array { return new ArraySub0(); });
        run("new ArraySub1(5), setter throws", function():Array { return new ArraySub1(5); });
        run("new ArraySub1(\"5\"), setter throws", function():Array { return new ArraySub1("5"); });
        run("new ArraySub3(1, 2, 3), setter throws", function():Array { return new ArraySub3(1, 2, 3); });
        throwFromSetter = false;
    }

    private static function run(name:String, f:Function):void {
        trace("/// " + name);
        try {
            var a:Array = f();
            trace("length: " + a.length);
            if (a.length <= 3) {
                for (var i:uint = 0; i < a.length; i++) {
                    trace("[" + i + "]: " + typeof a[i] + ", " + (i in a ? "present" : "hole"));
                }
            }
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
    }
}
}
