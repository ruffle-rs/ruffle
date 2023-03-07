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

interface BaseInterface {}
interface SubInterface {}

class SelfRef implements BaseInterface {
	static var field:SelfRef = new SelfRef();
	static const self_ref_const:SelfRef = make_it();
	
	static function make_it():SelfRef {
		var foo:SelfRef = new SelfRef();
		var other:BaseInterface = foo;
		return foo;
	}
	
	function self_ref_method(param:SelfRef): SelfRef {
		return param;
	}
}

class SelfRefSubClass implements SubInterface {
	static const self_ref_base:SelfRef = SelfRef.make_it();
	static const self_ref_sub_iface:SubInterface = new SelfRefSubClass();
}

trace("///var self_ref:SelfRef = new SelfRef();");
var self_ref:SelfRef = new SelfRef();
trace("/// self_ref");
trace(self_ref);

trace("///SelfRef.field");
trace(SelfRef.field);

trace("///SelfRef.self_ref_const");
trace(SelfRef.self_ref_const);

trace("///self_ref.self_ref_method(self_ref)");
trace(self_ref.self_ref_method(self_ref));

trace("///SelfRefSubClass.self_ref_base");
trace(SelfRefSubClass.self_ref_base);

trace("///SelfRefSubClass.self_ref_sub_iface");
trace(SelfRefSubClass.self_ref_sub_iface);

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