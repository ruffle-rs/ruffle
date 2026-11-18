package {
    import flash.display.Sprite;
    import flash.net.ObjectEncoding;
    import flash.net.registerClassAlias;
    import flash.utils.ByteArray;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            trace("== sealed Proxy ==");
            write(new SealedProxy());
            trace("== dynamic Proxy ==");
            write(new DynamicProxy());
            trace("== QName ==");
            write(new QName("urn:x", "n"));
            trace("== sealed Proxy with alias ==");
            registerClassAlias("SP", SealedProxy);
            write(new SealedProxy());
            trace("== sealed class with a public property ==");
            write(new SealedWithPublicProperty());
            write(new SealedWithPublicProperty(), ObjectEncoding.AMF0);
            trace("done");
        }

        private static function write(v:*, encoding:uint = 3):* {
            var b:ByteArray = new ByteArray();
            b.objectEncoding = encoding;
            b.writeObject(v);
            var s:Array = [];
            for (var i:int = 0; i < b.length; i++) {
                var h:String = b[i].toString(16);
                s.push(h.length == 1 ? "0" + h : h);
            }
            b.position = 0;
            var back:* = b.readObject();
            if (back is SealedProxy) {
                trace(s.join(" ") + " => SealedProxy");
                return back;
            }
            var keys:Array = [];
            for (var k:String in back) {
                keys.push(k + "=" + back[k]);
            }
            trace(s.join(" ") + " => {" + keys.join(",") + "}");
            return back;
        }
    }
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

class SealedWithPublicProperty {
    public var value:int = 42;
}

class SealedProxy extends Proxy {
    override flash_proxy function nextNameIndex(index:int):int {
        trace("  sealed nextNameIndex " + index);
        return index < 1 ? index + 1 : 0;
    }

    override flash_proxy function nextName(index:int):String {
        trace("  sealed nextName " + index);
        return "k";
    }

    override flash_proxy function nextValue(index:int):* {
        trace("  sealed nextValue " + index);
        return 1;
    }
}

dynamic class DynamicProxy extends Proxy {
    override flash_proxy function nextNameIndex(index:int):int {
        trace("  dynamic nextNameIndex " + index);
        return index < 1 ? index + 1 : 0;
    }

    override flash_proxy function nextName(index:int):String {
        trace("  dynamic nextName " + index);
        return "k";
    }

    override flash_proxy function nextValue(index:int):* {
        trace("  dynamic nextValue " + index);
        return 1;
    }
}
