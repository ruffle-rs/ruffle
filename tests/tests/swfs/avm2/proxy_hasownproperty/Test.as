package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var proxy: TestProxy = new TestProxy();
            trace('// proxy["hasOwnProperty"](123)');
            trace(proxy["hasOwnProperty"](123));

            trace('// proxy.hasOwnProperty("foobar")');
            trace(proxy.hasOwnProperty("foobar"));

            trace('// Object.prototype.hasOwnProperty.call(proxy, "hello world")');
            trace(Object.prototype.hasOwnProperty.call(proxy, "hello world"));
        }
    }
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
    override flash_proxy function hasProperty(name:*):Boolean {
        trace("hasProperty: " + typeof name + " " + name);
        return name == "foobar";
    }
}