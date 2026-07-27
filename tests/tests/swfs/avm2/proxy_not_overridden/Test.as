package {
import flash.display.Sprite;
import flash.utils.flash_proxy;

public class Test extends Sprite {
    public function Test() {
        var p:UnimplementedProxy = new UnimplementedProxy();

        test("getProperty", function():void { p.flash_proxy::getProperty("x"); });
        test("setProperty", function():void { p.flash_proxy::setProperty("x", 1); });
        test("callProperty", function():void { p.flash_proxy::callProperty("x"); });
        test("hasProperty", function():void { p.flash_proxy::hasProperty("x"); });
        test("deleteProperty", function():void { p.flash_proxy::deleteProperty("x"); });
        test("getDescendants", function():void { p.flash_proxy::getDescendants("x"); });
        test("nextNameIndex", function():void { p.flash_proxy::nextNameIndex(0); });
        test("nextName", function():void { p.flash_proxy::nextName(0); });
        test("nextValue", function():void { p.flash_proxy::nextValue(0); });
    }

    private function test(name:String, fn:Function):void {
        try {
            fn();
            trace(name + ": Not thrown");
        } catch (e:Error) {
            trace(name + ": " + e.getStackTrace());
        }
    }
}
}

import flash.utils.Proxy;

dynamic class UnimplementedProxy extends Proxy {
}
