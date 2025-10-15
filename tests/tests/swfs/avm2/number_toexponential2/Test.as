package {

import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        test(0.0);
        test(-0.0);
        test(Number.POSITIVE_INFINITY);
        test(Number.NEGATIVE_INFINITY);
        test(Number.NaN);
    }

    private function test(n: Number):void {
        trace(n);
        trace("  " + n.toExponential(0));
        trace("  " + n.toExponential(1));
        trace("  " + n.toExponential(2));
        trace("  " + n.toExponential(4));
        trace("  " + n.toExponential(10));
        trace("  " + n.toExponential(20));
    }
}

}
