package {

import flash.display.Sprite;

// Array.sort in SWF<11 has an observable behavior where you can differentiate
// between ints and floats.
public class Test extends Sprite {
    public function Test() {
        checkOrder([getA1() + getA2(), getB1() + getB2()]);
    }

    private function checkOrder(nums:Array) {
        nums.sort(Array.NUMERIC);
        trace(nums);
    }

    private function getA1(): Number {
        return 87000000.5;
    }

    private function getA2(): Number {
        return -0.5;
    }

    private function getB1(): Number {
        return -245000000.5;
    }

    private function getB2(): Number {
        return 0.5;
    }
}

}
