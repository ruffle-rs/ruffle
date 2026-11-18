package {
    import flash.display.Sprite;
    import flash.net.ObjectEncoding;
    import flash.net.registerClassAlias;
    import flash.utils.ByteArray;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            var list:XMLList = <><a/><b/></>;
            var cases:Array = [
                ["empty", new XMLList()],
                ["single", new XMLList("<a/>")],
                ["multiple", list],
                ["nested in object", {list: list}]
            ];
            for each (var encoding:uint in [ObjectEncoding.AMF3, ObjectEncoding.AMF0]) {
                trace("== AMF" + encoding + " ==");
                for each (var c:Array in cases) {
                    var b:ByteArray = new ByteArray();
                    b.objectEncoding = encoding;
                    b.writeObject(c[1]);
                    b.position = 0;
                    var back:* = b.readObject();
                    trace(c[0] + ": " + hex(b) + " => " + describe(back));
                }
            }

            trace("== AMF3 with alias ==");
            registerClassAlias("XL", XMLList);
            var aliased:ByteArray = new ByteArray();
            aliased.objectEncoding = ObjectEncoding.AMF3;
            aliased.writeObject(new XMLList("<a/>"));
            aliased.position = 0;
            var back:* = aliased.readObject();
            trace("aliased: " + hex(aliased) + " => " + getQualifiedClassName(back) + " length=" + back.length());
        }

        private static function hex(b:ByteArray):String {
            var s:Array = [];
            for (var i:int = 0; i < b.length; i++) {
                var h:String = b[i].toString(16);
                s.push(h.length == 1 ? "0" + h : h);
            }
            return s.join(" ");
        }

        private static function describe(v:*):String {
            var keys:Array = [];
            for (var k:String in v) {
                var inner:* = v[k];
                keys.push(k + "=" + getQualifiedClassName(inner));
            }
            keys.sort();
            return getQualifiedClassName(v) + " {" + keys.join(",") + "}";
        }
    }
}
