package {
	public class Test {
	}
}

namespace ruffle = "https://ruffle.rs/AS3/test_ns";

class First {
	var prop1:Second;
}

class Second {
	var prop2:First;
}

var any_var:* = undefined;
trace("///any_var");
trace(any_var);

var object_var:Object = undefined;
trace("///object_var");
trace(object_var);

trace("///var integer_var:int = 1.5");
var integer_var:int = 1.5;
trace("///integer_var");
trace(integer_var);
trace("///integer_var = 6.7;");
integer_var = 6.7;
trace(integer_var);
trace

var first:First = new First();
var second:Second = new Second();

trace("///first.prop1");
trace(first.prop1);
trace("///second.prop2");
trace(second.prop2);

trace("///first.prop1 = second;");
first.prop1 = second;
trace("///second.prop2 = first");
second.prop2 = first;

trace("///first.prop1");
trace(first.prop1);
trace("///second.prop2");
trace(second.prop2);

trace("//first.prop1 = new Object();");
first.prop1 = new Object();

trace("ERROR: This should be unreachable due to error being thrown");