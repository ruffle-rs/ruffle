package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function hasProperty(name:*):Boolean {
		trace("///attempted to check if has property:", name);
		
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

	var normal_var = "This is a normal var";
	
	flash_proxy var proxy_var = "This is a normal var in the proxy ns";
}

namespace my_ns = "my_ns";

var p = new TestProxy();

trace("///p.flash_proxy::hasProperty(\"via direct call\")");
trace(p.flash_proxy::hasProperty("via direct call"));

trace("///Object.prototype.hasOwnProperty.call(p, \"via_getproperty\")");
trace(Object.prototype.hasOwnProperty.call(p, "via_getproperty"));

trace("///\"via_getproperty\" in p");
trace("via_getproperty" in p);

trace("///new QName(my_ns, \"via_namespace\") in p");
trace(new QName(my_ns, "via_namespace") in p);

trace("///Object.prototype.hasOwnProperty.call(p, \"normal_var\")");
trace(Object.prototype.hasOwnProperty.call(p, "normal_var"));

trace("///\"normal_var\" in p");
trace("normal_var" in p);

trace("///Object.prototype.hasOwnProperty.call(p, new QName(flash_proxy, \"proxy_var\"))");
trace(Object.prototype.hasOwnProperty.call(p, new QName(flash_proxy, "proxy_var")));

trace("///new QName(flash_proxy, \"proxy_var\") in p");
trace(new QName(flash_proxy, "proxy_var") in p);

trace("///Object.prototype.hasOwnProperty.call(p, \"proxy_var\")");
trace(Object.prototype.hasOwnProperty.call(p, "proxy_var"));

trace("///\"proxy_var\" in p");
trace("proxy_var" in p);