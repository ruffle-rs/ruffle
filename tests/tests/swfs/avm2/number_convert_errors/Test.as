package {

import flash.display.*;

internal class FailingValueOf extends Object {
    public function valueOf():* {
        throw new Error("Failed");
    }
}

[SWF(width="200", height="200")]
public class Test extends MovieClip {
    public function Test() {
        var nums = [
            1,
            -1,
            123,
        ];
        var ps = [
            -1,
            0,
            1,
            20,
            21,
            22,
            28,
        ];

        for each (var num in nums) {
            for each (var p in ps) {
                var vp = tryCatch(function() { return Number(num).toPrecision(p); });
                var vf = tryCatch(function() { return Number(num).toFixed(p); });
                var ve = tryCatch(function() { return Number(num).toExponential(p); });
                trace(num + " (Number.toPrecision, " + p + ") -> " + vp);
                trace(num + " (Number.toFixed, " + p + ") -> " + vf);
                trace(num + " (Number.toExponential, " + p + ") -> " + ve);

                var pi = tryCatch(function() { return int(num).toPrecision(p); });
                var pu = tryCatch(function() { return uint(num).toPrecision(p); });
                trace(num + " (int.toPrecision, " + p + ") -> " + pi);
                trace(num + " (uint.toPrecision, " + p + ") -> " + pu);

                var fi = tryCatch(function() { return int(num).toFixed(p); });
                var fu = tryCatch(function() { return uint(num).toFixed(p); });
                trace(num + " (int.toFixed, " + p + ") -> " + fi);
                trace(num + " (uint.toFixed, " + p + ") -> " + fu);

                var ei = tryCatch(function() { return int(num).toFixed(p); });
                var eu = tryCatch(function() { return uint(num).toFixed(p); });
                trace(num + " (int.toFixed, " + p + ") -> " + ei);
                trace(num + " (uint.toFixed, " + p + ") -> " + eu);
            }
        }

        testCoercionErrors();
    }

    private function tryCatch(f:Function):* {
        try {
            return f();
        } catch (e) {
            return e.getStackTrace();
        }
    }

    private function testCoercionErrors():void {
        trace("coercion errors");

        var num = 0;

        var fNumber = tryCatch(function() { return Number(num).toFixed(new FailingValueOf()); });
        var fInt = tryCatch(function() { return int(num).toFixed(new FailingValueOf()); });
        var fUint = tryCatch(function() { return uint(num).toFixed(new FailingValueOf()); });

        var pNumber = tryCatch(function() { return Number(num).toPrecision(new FailingValueOf()); });
        var pInt = tryCatch(function() { return int(num).toPrecision(new FailingValueOf()); });
        var pUint = tryCatch(function() { return uint(num).toPrecision(new FailingValueOf()); });

        var eNumber = tryCatch(function() { return Number(num).toExponential(new FailingValueOf()); });
        var eInt = tryCatch(function() { return int(num).toExponential(new FailingValueOf()); });
        var eUint = tryCatch(function() { return uint(num).toExponential(new FailingValueOf()); });

        trace("coercion error (Number.toPrecision) -> " + pNumber);
        trace("coercion error (Number.toFixed) -> " + fNumber);
        trace("coercion error (Number.toExponential) -> " + eNumber);
        trace("coercion error (int.toPrecision) -> " + pInt);
        trace("coercion error (int.toFixed) -> " + fInt);
        trace("coercion error (int.toExponential) -> " + eInt);
        trace("coercion error (uint.toPrecision) -> " + pUint);
        trace("coercion error (uint.toFixed) -> " + fUint);
        trace("coercion error (uint.toExponential) -> " + eUint);
    }
}

}
