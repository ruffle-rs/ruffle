package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var vectors:Array = [
            Vector.<int>([1, 2, 3]),
            Vector.<uint>([1, 2, 3]),
            Vector.<Number>([1, 2, 3]),
            Vector.<Object>([1, 2, 3]),
            Vector.<String>(["1", "2", "3"])
        ];
        var names:Array = ["int", "uint", "Number", "Object", "String"];
        for (var i:int = 0; i < vectors.length; i++) {
            trace("///// Vector.<" + names[i] + ">");
            testVector(vectors[i]);
        }
    }

    private static function testVector(v:*):void {
        var receiver:Object = {};
        var cls:Class = Object(v).constructor;

        run("AS3::some", function():* { return v.some(callback(v, receiver, 2)); });
        run("public::some", function():* { return v.public::some(callback(v, receiver, 2)); });
        run("public::some, receiver", function():* { return v.public::some(callback(v, receiver, 2), receiver); });
        run("public::some, no match", function():* { return v.public::some(callback(v, receiver, 5)); });
        run("public::some, null", function():* { return v.public::some(null); });
        run("prototype.some.call", function():* { return cls.prototype.some.call(v, callback(v, receiver, 3)); });
    }

    private static function callback(v:*, receiver:Object, target:*):Function {
        return function(item:*, index:int, vector:*):Boolean {
            trace("callback(" + item + ", " + index + ", same vector: " + (vector === v) + ", this is receiver: " + (this === receiver) + ")");
            return item == target;
        };
    }

    private static function run(name:String, f:Function):void {
        trace("/// " + name);
        try {
            trace("result: " + f());
        } catch (e:*) {
            trace("caught: " + Object(e).constructor);
            trace(e.getStackTrace());
        }
    }
}
}
