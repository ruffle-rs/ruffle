package {
import flash.display.Sprite;

public class Test extends Sprite {
    private static var held:Error;

    public function Test() {
        var depths:Array = [62, 63, 64, 100];
        for each (var depth:int in depths) {
            trace("/// Error created at depth " + depth);
            show(rec(depth, newError));
        }

        depths = [60, 61];
        for each (depth in depths) {
            trace("/// Error created through Function.call at depth " + depth);
            show(rec(depth, newErrorViaCall));
        }

        trace("/// TypeError created at depth 100");
        show(rec(100, newTypeError));

        trace("/// Error subclass created at depth 100");
        show(rec(100, newCustomError));

        trace("/// Error created at depth 100, thrown at depth 0");
        var error:Error = rec(100, newError);
        try {
            throw error;
        } catch (e:*) {
            show(e);
        }

        trace("/// Error created at depth 0, thrown at depth 100");
        held = new Error("held");
        try {
            rec(100, throwHeld);
        } catch (e:*) {
            show(e);
        }

        trace("/// Runtime error at depth 100");
        try {
            rec(100, callNonFunction);
        } catch (e:*) {
            show(e);
        }
    }

    private static function show(e:*):void {
        trace(Object(e).constructor);
        trace(e.getStackTrace());
    }

    private static function rec(n:int, bottom:Function):* {
        if (n > 1) {
            return rec(n - 1, bottom);
        }
        return bottom();
    }

    private static function newError():* {
        return new Error("created");
    }

    private static function newErrorViaCall():* {
        return newError.call(null);
    }

    private static function newTypeError():* {
        return new TypeError("created");
    }

    private static function newCustomError():* {
        return new CustomError();
    }

    private static function throwHeld():* {
        throw held;
    }

    private static function callNonFunction():* {
        var o:* = 1;
        return o();
    }
}
}

class CustomError extends Error {
    public function CustomError() {
        super("custom");
    }
}
