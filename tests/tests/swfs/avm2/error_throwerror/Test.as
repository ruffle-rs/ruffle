package {
import flash.display.*;
import flash.errors.*;

public class Test extends Sprite {
    public function Test() {
        test(Error, 12);
        test(Error, 15);
        test(IOError, 1234);
        test(ReferenceError, 1074, "prototype", "MethodClosure");
        test(Error, 1075, "x");
        test(Error, 1044);
        test(Error, 1044, "a");
        test(Error, 1044, "a", "b");
        test(Error, 1044, "a", "b", "c");
        test(Error, 3723);
        test(Error, 3723, "a", "b", "c", "d", "e", "f", "g", "h", "i", "j");
        test(Error, 3683, "a", "b");
        test(Error, 3683, "%1");
        test(Error, 3683, "%2", "b");
        test(Error, 3683, null);
        test(Error, 3683, new Object());
        test(Error, 3683, 6);
        test(Error, 3683, new TrojanHorse());
        test(CustomError, 3683, "x");
    }

    private function test(type:Class, index:uint, ...rest): void {
        try {
            Error.throwError.apply(Error, [type, index].concat(rest));
            trace("Not thrown");
        } catch (e) {
            trace("Caught: " + e.getStackTrace());
        }
    }
}
}
