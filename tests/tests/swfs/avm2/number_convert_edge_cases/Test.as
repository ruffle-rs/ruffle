package {

import flash.display.*;
import flash.utils.getQualifiedClassName;

[SWF(width="200", height="200")]
public class Test extends MovieClip {
    public function Test() {
        var nums = [
            0.0/0.0,
            1.0/0.0,
            -1.0/0.0,
            0.0,
            -0.0,
            123,
        ];
        var digits = [
            undefined,
            null,
            0, 1, 2, 3,
            19, 20, 21, 22,
        ];

        for each (var num in nums) {
            for each (var i in digits) {
                var p = tryCatch(function() { return num.toPrecision(i); });
                var f = tryCatch(function() { return num.toFixed(i); });
                var e = tryCatch(function() { return num.toExponential(i); });
                trace(num + " (toPrecision,   " + i + ") -> " + p);
                trace(num + " (toFixed,       " + i + ") -> " + f);
                trace(num + " (toExponential, " + i + ") -> " + e);
            }
        }
    }

    private function tryCatch(f:Function):* {
        try {
            return f();
        } catch (e) {
            return "" + e;
        }
    }
}

}
