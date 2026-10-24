package {
import flash.display.Sprite;

public class Test extends Sprite {
    private var entered:int = 0;
    private var catches:int = 0;
    private var finallies:int = 0;

    public function Test() {
        run("plain", plain);
        run("plain again", plain);
        run("via call", viaCall);
        run("via apply", viaApply);
        run("via closure", viaClosure);
        run("via setter", function(n:int):void { viaSetter = n; });
        run("via getter", function(n:int):void { var v:int = viaGetter; });
        run("via constructor", function(n:int):void { new Node(); });
        run("via forEach", viaForEach);
        run("via toString", viaToString);

        trace("// catch at every level");
        entered = catches = 0;
        catchEveryLevel();
        trace("catches: " + catches);

        trace("// finally at every level");
        entered = finallies = 0;
        try {
            finallyEveryLevel();
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
        }
        trace("finally ran at every level: " + (finallies == entered));

        trace("// done");
    }

    private function run(name:String, f:Function):void {
        trace("// " + name);
        try {
            f(0);
            trace("no error");
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace("errorID: " + Object(e).errorID);
            trace(Object(e).getStackTrace().split("\n")[0]);
        }
    }

    private function plain(n:int):void {
        plain(n + 1);
    }

    private function viaCall(n:int):void {
        viaCall.call(this, n + 1);
    }

    private function viaApply(n:int):void {
        viaApply.apply(this, [n + 1]);
    }

    private function viaClosure(n:int):void {
        var self:Function = function(m:int):void {
            self(m + 1);
        };
        self(n);
    }

    private function set viaSetter(n:int):void {
        viaSetter = n + 1;
    }

    private function get viaGetter():int {
        return viaGetter + 1;
    }

    private function viaForEach(n:int):void {
        var arr:Array = [n + 1];
        arr.forEach(function(m:int, i:int, a:Array):void {
            viaForEach(m);
        });
    }

    private function viaToString(n:int):void {
        var o:Object = {};
        o.toString = function():String {
            viaToString(n + 1);
            return "";
        };
        String(o);
    }

    private function catchEveryLevel():void {
        entered++;
        try {
            catchEveryLevel();
        } catch (e:*) {
            if (catches == 0) {
                trace("caught: " + Object(e).constructor);
            }
            catches++;
        }
    }

    private function finallyEveryLevel():void {
        entered++;
        try {
            finallyEveryLevel();
        } finally {
            finallies++;
        }
    }
}
}

class Node {
    public function Node() {
        new Node();
    }
}
