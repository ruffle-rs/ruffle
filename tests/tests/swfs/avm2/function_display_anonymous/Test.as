package {

import flash.display.*;

public class Test extends Sprite {
    public function Test() {
        try {
            (function():void {
                throw new Error();
            })();
        } catch (e) {
            trace(e.getStackTrace());
        }

        try {
            (function(a:String):void {})();
        } catch (e) {
            trace(e.getStackTrace());
        }

        try {
            (function(a:String, b:String):void {})("a", "b", "c");
        } catch (e) {
            trace(e.getStackTrace());
        }
    }
}

}
