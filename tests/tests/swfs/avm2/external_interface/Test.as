package {
	public class Test {}
}

import flash.external.ExternalInterface;

class Target {
	function parrot() {
		trace("/// parrot() start");
		
		trace("// this");
		trace(this);
		trace("");
		
		trace("// this == thisTarget");
		trace(this == thisTarget);
		trace("");
		
		trace("// arguments.length");
		trace(arguments.length);
		trace("");
		
		trace("// arguments[0]");
		trace(arguments[0]);
		trace("");
		
		trace("/// parrot() end");
		return arguments[0];
	}

	function callWith(name, payload) {
		trace("/// callWith() start");
		
		trace("// this");
		trace(this);
		trace("");
		
		trace("// this == thatTarget");
		trace(this == thatTarget);
		trace("");
		
		trace("// arguments.length");
		trace(arguments.length);
		trace("");
		
		trace("// name");
		trace(name);
		trace("");
		
		trace("// payload");
		trace(payload);
		trace("");
		
		trace("// ExternalInterface.call(name, payload)");
		trace(ExternalInterface.call(name, payload));
		trace("");
		
		trace("/// callWith() end");
	}
}

function freestanding() {
	trace("/// freestanding() start");

	trace("// this");
	trace(this);
	trace("");
	
	trace("/// freestanding() end");
}

var thisTarget = new Target();
var thatTarget = new Target();

trace("// ExternalInterface.available");
trace(ExternalInterface.available);
trace("");

trace("// ExternalInterface.addCallback(\"dump\", thisTarget.dump)");
ExternalInterface.addCallback("dump", thisTarget.dump);
trace("");

trace("// ExternalInterface.addCallback(\"parrot\", thisTarget.parrot)");
ExternalInterface.addCallback("parrot", thisTarget.parrot);
trace("");

trace("// ExternalInterface.addCallback(\"callWith\", thatTarget.callWith)");
ExternalInterface.addCallback("callWith", thatTarget.callWith);
trace("");

trace("// ExternalInterface.addCallback(\"freestanding\", freestanding)");
ExternalInterface.addCallback("freestanding", freestanding);
trace("");

trace("// ExternalInterface.call(\"ping\")");
trace(ExternalInterface.call("ping"));
trace("");

trace("// ExternalInterface.call(\"non_existent\")");
trace(ExternalInterface.call("non_existent"));
trace("");

trace("// ExternalInterface.call(\"reentry\")");
trace(ExternalInterface.call("reentry"));
trace("");
