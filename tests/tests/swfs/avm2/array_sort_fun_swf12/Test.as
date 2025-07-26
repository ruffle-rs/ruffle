package {
import flash.display.*;

// See <https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/ArrayClass.cpp#L821>
// See <https://bugzilla.mozilla.org/show_bug.cgi?id=532454>
public class Test extends MovieClip {
    public function Test() {
        var compareA:Function = function(a:Number, b:Number):Number {
            return a - b;
        };

        var compareB:Function = function(a:Number, b:Number):Number {
            if (a < b) return -1;
            if (a > b) return 1;
            return 0;
        };

        var nums = [2.1, 2.0, 1.9, 1.1, 0.9, 0, -0.1, -0.5, -0.9, -1.0, -1.1, -1.5, -1.9, -2.0, -2.1];
        trace(nums.sort(compareA));
        trace(nums.sort(compareB));
    }
}
}
