package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function getProperty(name:*):* {
		trace("///attempted to get property: ", name);
		return this.flash_proxy::proxy_fn;
	}
	
	flash_proxy override function callProperty(name:*, ... rest):* {
		trace("///attempted to call property: ", name);
		
		if (name is QName) {
			trace("///type of name is QName");
			
			trace("///name.localName");
			trace(name.localName);
			
			trace("///name.uri");
			trace(name.uri);
		} else if (name is String) {
			trace("///type of name is String");
		} else {
			trace("///invalid name type");
		}
		
		return null;
	}

	function normal_fn() {
		trace("///called normal_fn");
	}
	
	flash_proxy function proxy_fn() {
		trace("///called proxy_fn");
	}
}

namespace my_ns = "my_ns";

var p = new TestProxy();

p.callProperty("via direct call");
p.via_callproperty();
p.my_ns::via_namespace();
p.normal_fn();
p.proxy_fn();