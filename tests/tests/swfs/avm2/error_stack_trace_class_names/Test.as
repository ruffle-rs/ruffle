package {
import flash.display.Sprite;
import pkg.Pub;
import pkg.custom_ns;

public class Test extends Sprite {
    public function Test() {
        var t:Test = this;
        run("Test static", function():void { staticMethod(); });
        run("Test instance", function():void { t.instanceMethod(); });
        run("Test private", function():void { t.privateMethod(); });
        run("Test protected", function():void { t.protectedMethod(); });
        run("Test internal", function():void { t.internalMethod(); });
        run("Test custom namespace", function():void { t.custom_ns::namespacedMethod(); });
        run("Test getter", function():void { var x:int = t.getter; });
        run("Test setter", function():void { t.setter = 1; });

        run("public class constructor", function():void { new Pub(true); });
        run("public class static", function():void { Pub.staticMethod(); });
        run("public class instance", function():void { new Pub().instanceMethod(); });
        run("public class getter", function():void { var x:int = new Pub().getter; });

        run("internal class constructor", function():void { Pub.makeHidden(true); });
        run("internal class static", function():void { Pub.hiddenStatic(); });
        run("internal class instance", function():void { Pub.makeHidden(false).instanceMethod(); });

        run("private class constructor", function():void { new Priv(true); });
        run("private class static", function():void { Priv.staticMethod(); });
        run("private class instance", function():void { new Priv(false).instanceMethod(); });
        run("private class getter", function():void { var x:int = new Priv(false).getter; });
    }

    public static function staticMethod():void {
        throw new Error("thrown");
    }

    public function instanceMethod():void {
        throw new Error("thrown");
    }

    private function privateMethod():void {
        throw new Error("thrown");
    }

    protected function protectedMethod():void {
        throw new Error("thrown");
    }

    internal function internalMethod():void {
        throw new Error("thrown");
    }

    custom_ns function namespacedMethod():void {
        throw new Error("thrown");
    }

    public function get getter():int {
        throw new Error("thrown");
    }

    public function set setter(value:int):void {
        throw new Error("thrown");
    }

    private static function run(name:String, f:Function):void {
        trace("/// " + name);
        try {
            f();
            trace("no error");
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
    }
}
}

class Priv {
    public function Priv(fail:Boolean) {
        if (fail) {
            throw new Error("thrown");
        }
    }

    public static function staticMethod():void {
        throw new Error("thrown");
    }

    public function instanceMethod():void {
        throw new Error("thrown");
    }

    public function get getter():int {
        throw new Error("thrown");
    }
}
