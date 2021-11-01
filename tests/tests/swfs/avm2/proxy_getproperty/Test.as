package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function getProperty(name:*):* {
		trace("///attempted to get property:", name);
		
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
		
		return "This is a proxy var";
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

trace("///p.flash_proxy::getProperty(\"via direct call\")");
trace(p.flash_proxy::getProperty("via direct call"));

trace("///p.via_getproperty");
trace(p.via_getproperty);

trace("///p.my_ns::via_namespace");
trace(p.my_ns::via_namespace);

trace("///p.normal_var");
trace(p.normal_var);

trace("///p.flash_proxy::proxy_var");
trace(p.flash_proxy::proxy_var);

trace("///(p.normal_fn)()");
trace((p.normal_fn)());

trace("///p.flash_proxy::proxy_fn");
trace(p.flash_proxy::proxy_fn);

trace("///p.proxy_var");
trace(p.proxy_var);

trace("///p.proxy_fn");
trace(p.proxy_fn);