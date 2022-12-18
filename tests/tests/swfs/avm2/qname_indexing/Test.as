// compiled with mxmlc

import flash.utils.Proxy;
import flash.utils.flash_proxy;

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

namespace ns1 = "ns1";
namespace ns2 = "ns2";

class C {
	public var x;
	ns1 var y;
	ns2 var y;
}
var c = new C();
c.x = 1;
c.ns1::y = 2;
c.ns2::y = 3;

trace("basic indexing")
trace(c.x);
trace(c["x"]);
trace(c[new QName("x")]);

trace()
trace("qname with namespace")
trace(c[new QName(ns1, "y")]);
trace(c[new QName("ns1", "y")]);

trace()
trace("namespace part is ignored")
trace(c.ns2::[new QName("ns1", "y")]);
trace(c.ns1::[new QName("ns2", "y")]);

dynamic class MyProxy extends Proxy {
    flash_proxy override function getProperty(name:*):* {
    	trace(getQualifiedClassName(name));
    	trace(name);
    	return c[name];
    }
}
trace()
trace("through a proxy")
var proxy = new MyProxy();
trace(proxy.ns1::y);
trace(proxy.ns2::y);
trace(proxy.x);
