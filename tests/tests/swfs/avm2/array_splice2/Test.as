package {
import flash.display.*;

public class Test extends MovieClip {
    public function Test() {
        var array_sparse1:Array = [];
        array_sparse1[100000] = 5;

        var array_sparse2:Array = [1];
        array_sparse2[100000] = 5;

        var arrays:Array = [
            [],
            [1],
            [1, 2, 3, 4, 5],
            array_sparse1,
            array_sparse2,
            [[1], [1, 2], [1, 2, 3]],
            [1, 2, [3], 4, [5], 6],
        ];

        var i:int = 0;
        while (i < arrays.length) {
            testArray(i, arrays[i]);
            ++i;
        }
    }

    private function testArray(i:int, array:Array):void {
        trace("Testing array " + i + ":");
        trace("  array: " + arrToString(array));

        trace("  splice():");
        testSplice(array, function(a:Array):Array {
            return a.splice();
        });

        trace("  splice(0):");
        testSplice(array, function(a:Array):Array {
            return a.splice(0);
        });

        trace("  splice(1):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1);
        });

        trace("  splice(a.length):");
        testSplice(array, function(a:Array):Array {
            return a.splice(a.length);
        });

        trace("  splice(a.length-1):");
        testSplice(array, function(a:Array):Array {
            return a.splice(a.length-1);
        });

        trace("  splice(a.length+1):");
        testSplice(array, function(a:Array):Array {
            return a.splice(a.length+1);
        });

        trace("  splice(a.length/2):");
        testSplice(array, function(a:Array):Array {
            return a.splice(a.length/2);
        });

        trace("  splice(1,2):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1,2);
        });

        trace("  splice(1,2,3,4):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1,2,3,4);
        });

        trace("  splice(1,2,3,4,5,6):");
        testSplice(array, function(a:Array):Array {
            return a.splice(1,2,3,4,5,6);
        });
    }

    private function testSplice(array:Array, f:Function):void {
        array = array.concat();
        var ret:Array = f(array);

        trace("    returned: " + arrToString(ret));
        trace("    after: " + arrToString(array));
        for (var i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 99999, 100000, 100001]) {
            if (ret != null && ret[i] != null) {
                trace("    returned[" + i + "]: " + ret[i]);
            }
            if (array != null && array[i] != null) {
                trace("    after[" + i + "]: " + array[i]);
            }
        }
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
