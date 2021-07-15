package {
	public class Test {
	}
}

class CoercibleAsIntString {
	public function toString() {
		return "-99.13";
	}
}

class CoercibleAsNonIntString {
	public function toString() {
		return "TEST FAIL";
	}
}

class CoercibleAsValue {
	public function valueOf() {
		return 23.16;
	}
}

class NotCoercibleAsValue {
	public function valueOf() {
		return "TEST FAIL";
	}
}

trace("// == int tests == ");
trace("//undefined is int");
trace(undefined is int);
trace("//null is int");
trace(null is int);
trace("//true is int");
trace(true is int);
trace("//false is int");
trace(false is int);
trace("//0 is int");
trace(0 is int);
trace("//1 is int");
trace(1 is int);
trace("//5.12 is int");
trace(5.12 is int);
trace("//(5.12 - .12) is int");
trace((5.12 - .12) is int);
trace("//-6 is int");
trace(-6 is int);
trace("//\"12.23\" is int");
trace("12.23" is int);
trace("//\"true\" is int");
trace("true" is int);
trace("//\"false\" is int");
trace("false" is int);
trace("//new CoercibleAsIntString() is int");
trace(new CoercibleAsIntString() is int);
trace("//new CoercibleAsNonIntString() is int");
trace(new CoercibleAsNonIntString() is int);
trace("//new CoercibleAsValue() is int");
trace(new CoercibleAsValue() is int);
trace("//new NotCoercibleAsValue() is int");
trace(new NotCoercibleAsValue() is int);

trace("// == uint tests == ");
trace("//undefined is uint");
trace(undefined is uint);
trace("//null is uint");
trace(null is uint);
trace("//true is uint");
trace(true is uint);
trace("//false is uint");
trace(false is uint);
trace("//0 is uint");
trace(0 is uint);
trace("//1 is uint");
trace(1 is uint);
trace("//5.12 is uint");
trace(5.12 is uint);
trace("//(5.12 - .12) is uint");
trace((5.12 - .12) is uint);
trace("//-6 is uint");
trace(-6 is uint);
trace("//\"12.23\" is uint");
trace("12.23" is uint);
trace("//\"true\" is uint");
trace("true" is uint);
trace("//\"false\" is uint");
trace("false" is uint);
trace("//new CoercibleAsIntString() is uint");
trace(new CoercibleAsIntString() is uint);
trace("//new CoercibleAsNonIntString() is uint");
trace(new CoercibleAsNonIntString() is uint);
trace("//new CoercibleAsValue() is uint");
trace(new CoercibleAsValue() is uint);
trace("//new NotCoercibleAsValue() is uint");
trace(new NotCoercibleAsValue() is uint);

trace("// == Number tests == ");
trace("//undefined is Number");
trace(undefined is Number);
trace("//null is Number");
trace(null is Number);
trace("//true is Number");
trace(true is Number);
trace("//false is Number");
trace(false is Number);
trace("//0 is Number");
trace(0 is Number);
trace("//1 is Number");
trace(1 is Number);
trace("//5.12 is Number");
trace(5.12 is Number);
trace("//(5.12 - .12) is Number");
trace((5.12 - .12) is Number);
trace("//-6 is Number");
trace(-6 is Number);
trace("//\"12.23\" is Number");
trace("12.23" is Number);
trace("//\"true\" is Number");
trace("true" is Number);
trace("//\"false\" is Number");
trace("false" is Number);
trace("//new CoercibleAsIntString() is Number");
trace(new CoercibleAsIntString() is Number);
trace("//new CoercibleAsNonIntString() is Number");
trace(new CoercibleAsNonIntString() is Number);
trace("//new CoercibleAsValue() is Number");
trace(new CoercibleAsValue() is Number);
trace("//new NotCoercibleAsValue() is Number");
trace(new NotCoercibleAsValue() is Number);

trace("// == Boolean tests == ");
trace("//undefined is Boolean");
trace(undefined is Boolean);
trace("//null is Boolean");
trace(null is Boolean);
trace("//true is Boolean");
trace(true is Boolean);
trace("//false is Boolean");
trace(false is Boolean);
trace("//0 is Boolean");
trace(0 is Boolean);
trace("//1 is Boolean");
trace(1 is Boolean);
trace("//5.12 is Boolean");
trace(5.12 is Boolean);
trace("//(5.12 - .12) is Boolean");
trace((5.12 - .12) is Boolean);
trace("//-6 is Boolean");
trace(-6 is Boolean);
trace("//\"12.23\" is Boolean");
trace("12.23" is Boolean);
trace("//\"true\" is Boolean");
trace("true" is Boolean);
trace("//\"false\" is Boolean");
trace("false" is Boolean);
trace("//new CoercibleAsIntString() is Boolean");
trace(new CoercibleAsIntString() is Boolean);
trace("//new CoercibleAsNonIntString() is Boolean");
trace(new CoercibleAsNonIntString() is Boolean);
trace("//new CoercibleAsValue() is Boolean");
trace(new CoercibleAsValue() is Boolean);
trace("//new NotCoercibleAsValue() is Boolean");
trace(new NotCoercibleAsValue() is Boolean);

trace("// == String tests == ");
trace("//undefined is String");
trace(undefined is String);
trace("//null is String");
trace(null is String);
trace("//true is String");
trace(true is String);
trace("//false is String");
trace(false is String);
trace("//0 is String");
trace(0 is String);
trace("//1 is String");
trace(1 is String);
trace("//5.12 is String");
trace(5.12 is String);
trace("//(5.12 - .12) is String");
trace((5.12 - .12) is String);
trace("//-6 is String");
trace(-6 is String);
trace("//\"12.23\" is String");
trace("12.23" is String);
trace("//\"true\" is String");
trace("true" is String);
trace("//\"false\" is String");
trace("false" is String);
trace("//new CoercibleAsIntString() is String");
trace(new CoercibleAsIntString() is String);
trace("//new CoercibleAsNonIntString() is String");
trace(new CoercibleAsNonIntString() is String);
trace("//new CoercibleAsValue() is String");
trace(new CoercibleAsValue() is String);
trace("//new NotCoercibleAsValue() is String");
trace(new NotCoercibleAsValue() is String);

trace("// == Object tests == ");
trace("//undefined is Object");
trace(undefined is Object);
trace("//null is Object");
trace(null is Object);
trace("//true is Object");
trace(true is Object);
trace("//false is Object");
trace(false is Object);
trace("//0 is Object");
trace(0 is Object);
trace("//1 is Object");
trace(1 is Object);
trace("//5.12 is Object");
trace(5.12 is Object);
trace("//(5.12 - .12) is Object");
trace((5.12 - .12) is Object);
trace("//-6 is Object");
trace(-6 is Object);
trace("//\"12.23\" is Object");
trace("12.23" is Object);
trace("//\"true\" is Object");
trace("true" is Object);
trace("//\"false\" is Object");
trace("false" is Object);
trace("//new CoercibleAsIntString() is Object");
trace(new CoercibleAsIntString() is Object);
trace("//new CoercibleAsNonIntString() is Object");
trace(new CoercibleAsNonIntString() is Object);
trace("//new CoercibleAsValue() is Object");
trace(new CoercibleAsValue() is Object);
trace("//new NotCoercibleAsValue() is Object");
trace(new NotCoercibleAsValue() is Object);