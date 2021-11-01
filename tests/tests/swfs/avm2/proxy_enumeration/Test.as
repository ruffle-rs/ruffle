package {
	public class Test {}
}

import flash.utils.Proxy;
import flash.utils.flash_proxy;

dynamic class TestProxy extends Proxy {
	flash_proxy override function nextNameIndex(index:int):int {
		trace("///attempted to get next name index for index:", index);
		
		if (index < 5) {
			return index + 1;
		} else {
			return 0;
		}
	}

	flash_proxy override function nextName(index:int):String {
		trace("///attempted to get name for index:", index);
		
		return "proxy key " + index;
	}
	
	flash_proxy override function nextValue(index:int):* {
		trace("///attempted to get value for index:", index);
		
		return "proxy value " + index;
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

trace("///for (var k in p)...");
for (var k in p) {
	trace(k);
}

trace("///for each (var k in p)...");
for each (var k in p) {
	trace(k);
}