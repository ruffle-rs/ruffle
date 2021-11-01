package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function setProperty(name:*, value:*):void {
		trace("///attempted to set property:", name, "to value:", value);
		
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
	}

	var normal_var = "This is a normal var";
	
	flash_proxy var proxy_var = "This is a normal var in the proxy ns";
}

namespace my_ns = "my_ns";

var p = new TestProxy();

trace("///p.flash_proxy::setProperty(\"via direct call\", \"and value\")");
trace(p.flash_proxy::setProperty("via direct call", "and value"));

trace("///p.via_setproperty = \"test\"");
trace(p.via_setproperty = "test");

trace("///p.my_ns::via_namespace = 123");
trace(p.my_ns::via_namespace = 123);

trace("///p.normal_var = \"Another var value\"");
trace(p.normal_var = "Another var value");

trace("///p.normal_var");
trace(p.normal_var);

trace("///p.flash_proxy::proxy_var = false");
trace(p.flash_proxy::proxy_var = false);

trace("///p.proxy_var = \"Another var value in the proxy ns\"");
trace(p.proxy_var = "Another var value in the proxy ns");