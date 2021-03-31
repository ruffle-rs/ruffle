package {
	public class Test {
	}
}

import flash.events.Event;

trace("///(DynEvent is a dynamic subclass of Event)");
dynamic class DynEvent extends Event {
	function DynEvent(type: String, bubbles: Boolean = false, cancelable: Boolean = false) {
		super(type, bubbles, cancelable);
		
		this.__evil = 0;
	}
	
	public function get evilProp() {
		var x = this.__evil;
		
		this.__evil += 1;
		
		return x;
	}
	
	public function get strProp() {
		return "strProp";
	}
	
	public function get numProp() {
		return 5;
	}
}

trace("///var e = new DynEvent(\"test_event\", false, true);");
var e = new DynEvent("test_event", false, true);

trace("///e.formatToString(\"MyClass\");");
trace(e.formatToString("MyClass"));

trace("///e.property = \"value\";");
e.property = "value";

trace("///e.formatToString(\"MyClass\", \"property\");");
trace(e.formatToString("MyClass", "property"));

trace("///e[2] = \"property\";");
e[2] = "property";

trace("///e.three = true;");
e.three = true;

trace("///e.four = 0.5;");
e.four = 0.5;

trace("///e.five = 10;");
e.five = 10;

trace("///e.six = NaN;");
e.six = NaN;

trace("///e.formatToString(\"MyClass\", 2);");
trace(e.formatToString("MyClass", 2));

trace("///e.formatToString(\"MyClass\", 2, \"property\");");
trace(e.formatToString("MyClass", 2, "property"));

trace("///e.formatToString(\"MyClass\", \"property\", 2, \"property\");");
trace(e.formatToString("MyClass", "property", 2, "property"));

trace("///e.formatToString(\"MyClass\", \"three\", \"four\", \"five\");");
trace(e.formatToString("MyClass", "three", "four", "five"));

trace("///e.formatToString(\"MyClass\", \"strProp\", \"numProp\");");
trace(e.formatToString("MyClass", "strProp", "numProp"));

trace("///e.formatToString(\"MyClass\", \"evilProp\", 2, \"evilProp\");");
trace(e.formatToString("MyClass", "evilProp", 2, "evilProp"));

trace("///DynEvent.prototype.protoProp = \"protoValue\"");
DynEvent.prototype.protoProp = "protoValue";

trace("///e.formatToString(\"MyClass\", \"protoProp\");");
trace(e.formatToString("MyClass", "protoProp"));

trace("///e.formatToString(\"MyClass\", undefined);");
trace(e.formatToString("MyClass", undefined));

trace("///e.formatToString(\"MyClass\", null);");
trace(e.formatToString("MyClass", null));