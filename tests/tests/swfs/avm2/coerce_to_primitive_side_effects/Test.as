// compiled with mxmlc

import flash.utils.getQualifiedClassName;
import flash.utils.Proxy;
import flash.utils.flash_proxy;

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
    }
}

class MyProxy extends Proxy {
    public var returnValue;
    public function MyProxy(val) { this.returnValue = val; }
    flash_proxy override function callProperty(name:*, ...rest):* {
        trace("call " + name);
        return this.returnValue;
    }
    flash_proxy override function getProperty(name:*):* {
        trace("get " + name);
        return undefined;
    }
    flash_proxy override function hasProperty(name:*):Boolean {
        trace("has " + name);
        return false;
    }
}

function test(obj){
    trace("----------- " + obj.returnValue);
    var s: String;
    var i: int;
    try {
        s = obj;
        trace(getQualifiedClassName(s));
        trace(s);
    } catch (e) { trace("Error " + e.errorID); }

    try {
        i = obj;
        trace(getQualifiedClassName(i));
        trace(i);
    } catch (e) { trace("Error " + e.errorID); }
}

function main(){
    test(new MyProxy("1234"));
    test(new MyProxy(undefined));
    test(new MyProxy(new Object()));
    trace("--- string null")
    var s: String = null;
    test(new MyProxy(s));
    // coerce_to_primitive_side_effects_with_nulls is the same test, with these uncommented
    // trace("--- normal null")
    // test(new MyProxy(null));
}
main();