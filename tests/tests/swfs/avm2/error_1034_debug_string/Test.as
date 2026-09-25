package {
    import flash.display.Sprite;
    import flash.geom.Point;
    import flash.events.IEventDispatcher;
    import flash.utils.Dictionary;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            probe("Date", new Date(2009, 1, 27, 3, 4, 5));
            probe("invalid Date", new Date(NaN));
            probe("Date class", Date);
            probe("Point class", Point);
            probe("int class", int);
            probe("Object class", Object);
            probe("interface", IEventDispatcher);
            probe("plain Object", {});
            probe("empty Array", []);
            probe("Array w/ elems", [1, 2, 3]);
            probe("Array subclass", new MyArray());
            probe("Array class", Array);
            probe("custom toString", new HasToString());
            probe("function literal", function():void {});
            probe("new Function()", new Function());
            probe("Function class", Function);
            probe("instance method", (new HasToString()).m);
            probe("static method", Test.sm);
            probe("Namespace", new Namespace("ns", "http://x"));
            probe("empty Namespace", new Namespace(""));
            probe("QName", new QName("http://x", "local"));
            probe("XML", new XML("<a b='c'>t</a>"));
            probe("XMLList", new XML("<a><b/><b/></a>").b);
            probe("RegExp", /ab+c/gi);
            probe("Error object", new TypeError("boom"));
            probe("Boolean object", new Boolean(true));
            probe("String object", new String("hi"));
            probe("Dictionary", new Dictionary());
            probe("ByteArray", new ByteArray());
            probe("Sprite (this)", this);
        }

        public static function sm():void {}

        private function probe(label:String, v:*):void {
            try {
                var p:Point = v;
                trace(label + " || no throw: " + p);
            } catch (e:*) {
                var normalized = e.toString().replace(/@[0-9A-Fa-f]+/g, "@ADDRESS");
                trace(label + " || " + normalized);
            }
        }
    }
}

class HasToString {
    public function toString():String { return "I-am-custom"; }
    public function m():void {}
}

dynamic class MyArray extends Array {
}
