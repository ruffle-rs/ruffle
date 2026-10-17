package pkg {
public class Pub {
    public function Pub(fail:Boolean = false) {
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

    public static function makeHidden(fail:Boolean):Object {
        return new Hidden(fail);
    }

    public static function hiddenStatic():void {
        Hidden.staticMethod();
    }
}
}
