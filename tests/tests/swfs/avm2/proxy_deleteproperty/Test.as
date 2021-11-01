package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function deleteProperty(name:*):Boolean {
		trace("///attempted to delete property:", name);
		
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
		
		return true;
	}

	function normal_fn() {
		trace("///called normal_fn");
	}

	var normal_var = "This is a normal var";
	
	flash_proxy function proxy_fn() {
		trace("///called proxy_fn");
	}
	
	flash_proxy var proxy_var = "This is a normal var in the proxy ns";
}

namespace my_ns = "my_ns";

var p = new TestProxy();

trace("///p.flash_proxy::deleteProperty(\"via direct call\")");
trace(p.flash_proxy::deleteProperty("via direct call"));

trace("///delete p.via_getproperty");
trace(delete p.via_getproperty);

trace("///delete p.my_ns::via_namespace");
trace(delete p.my_ns::via_namespace);

trace("///delete p.normal_var");
trace(delete p.normal_var);

trace("///p.normal_var");
trace(p.normal_var);

trace("///delete p.flash_proxy::proxy_var");
trace(delete p.flash_proxy::proxy_var);

trace("///delete (p.normal_fn)()");
trace(delete (p.normal_fn)());

trace("///delete p.flash_proxy::proxy_fn");
trace(delete p.flash_proxy::proxy_fn);

trace("///delete p.proxy_var");
trace(delete p.proxy_var);

trace("///delete p.proxy_fn");
trace(delete p.proxy_fn);