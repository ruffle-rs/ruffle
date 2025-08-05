package {
import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        checkOrder([287000000,-273000000,-270000000,87000000,-245000000]);
    }

    private function checkOrder(nums:Array) {
        nums.sort(Array.NUMERIC);
        trace(nums);
    }
}
}
