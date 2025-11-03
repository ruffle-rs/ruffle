package {
import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        var array = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];

        trace("splice():");
        testSplice(array, function(a:Array):Array {
            return a.splice();
        });

        trace("splice(\"0\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice("0");
        });

        trace("splice(\"5\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice("5");
        });

        trace("splice(true):");
        testSplice(array, function(a:Array):Array {
            return a.splice(true);
        });

        trace("splice(false):");
        testSplice(array, function(a:Array):Array {
            return a.splice(false);
        });

        trace("splice(new Object()):");
        testSplice(array, function(a:Array):Array {
            return a.splice(new Object());
        });

        trace("splice(1, \"2\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, "2");
        });

        trace("splice(-1, 2):");
        testSplice(array, function(a:Array):Array {
            return a.splice(-1, 2);
        });

        trace("splice(\"-5\", 3):");
        testSplice(array, function(a:Array):Array {
            return a.splice("-5", 3);
        });

        trace("splice(1, -2):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, -2);
        });

        trace("splice(1, true):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, true);
        });

        trace("splice(1, false):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, false);
        });

        trace("splice(1, \"true\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, "true");
        });

        trace("splice(1, \"false\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, "false");
        });

        trace("splice(1, new Object()):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, new Object());
        });

        trace("splice(1, \"5\"):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1, "5");
        });
    }

    private function testSplice(array:Array, f:Function):void {
        array = array.concat();
        var ret:Array = f(array);

        trace("  returned: " + arrToString(ret));
        trace("  after: " + arrToString(array));
    }

    private function arrToString(array:*):String {
        if (array == null) {
            return "null";
        } else if (!(array is Array)) {
            return "" + array;
        } else if (array.length > 1000) {
            return "<len(" + array.length + ")>";
        } else if (array.length == 0) {
            return "<empty>";
        } else {
            return "[" + array.map(function(el:*, ...rest):String {
                return arrToString(el);
            }) + "]";
        }
    }
}
}
