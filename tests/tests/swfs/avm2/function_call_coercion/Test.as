package {
	public class Test {}
}

class CoercibleAsString {
	public function toString() {
		return "-99.13";
	}
}

class CoercibleAsValue {
	public function valueOf() {
		return 23.16;
	}
}

function coerces_param_into_int(i: int) {
	return i;
}

function coerces_param_into_uint(i: uint) {
	return i;
}

function coerces_param_into_Number(i: Number) {
	return i;
}

function coerces_param_into_Boolean(i: Boolean) {
	return i;
}

function coerces_param_into_String(i: String) {
	return i;
}

function coerces_param_into_Object(i: Object) {
	return i;
}

trace("//coerces_param_into_int(undefined);");
trace(coerces_param_into_int(undefined));

trace("//coerces_param_into_int(null);");
trace(coerces_param_into_int(null));

trace("//coerces_param_into_int(true);");
trace(coerces_param_into_int(true));

trace("//coerces_param_into_int(false);");
trace(coerces_param_into_int(false));

trace("//coerces_param_into_int(5.12);");
trace(coerces_param_into_int(5.12));

trace("//coerces_param_into_int(-6);");
trace(coerces_param_into_int(-6));

trace("//coerces_param_into_int(\"12.23\");");
trace(coerces_param_into_int("12.23"));

trace("//coerces_param_into_int(new CoercibleAsString());");
trace(coerces_param_into_int(new CoercibleAsString()));

trace("//coerces_param_into_int(new CoercibleAsValue());");
trace(coerces_param_into_int(new CoercibleAsValue()));

trace("//coerces_param_into_uint(undefined);");
trace(coerces_param_into_uint(undefined));

trace("//coerces_param_into_uint(null);");
trace(coerces_param_into_uint(null));

trace("//coerces_param_into_uint(true);");
trace(coerces_param_into_uint(true));

trace("//coerces_param_into_uint(false);");
trace(coerces_param_into_uint(false));

trace("//coerces_param_into_uint(5.12);");
trace(coerces_param_into_uint(5.12));

trace("//coerces_param_into_uint(-6);");
trace(coerces_param_into_uint(-6));

trace("//coerces_param_into_uint(\"12.23\");");
trace(coerces_param_into_uint("12.23"));

trace("//coerces_param_into_uint(new CoercibleAsString());");
trace(coerces_param_into_uint(new CoercibleAsString()));

trace("//coerces_param_into_uint(new CoercibleAsValue());");
trace(coerces_param_into_uint(new CoercibleAsValue()));

trace("//coerces_param_into_Number(undefined);");
trace(coerces_param_into_Number(undefined));

trace("//coerces_param_into_Number(null);");
trace(coerces_param_into_Number(null));

trace("//coerces_param_into_Number(true);");
trace(coerces_param_into_Number(true));

trace("//coerces_param_into_Number(false);");
trace(coerces_param_into_Number(false));

trace("//coerces_param_into_Number(5.12);");
trace(coerces_param_into_Number(5.12));

trace("//coerces_param_into_Number(-6);");
trace(coerces_param_into_Number(-6));

trace("//coerces_param_into_Number(\"12.23\");");
trace(coerces_param_into_Number("12.23"));

trace("//coerces_param_into_Number(new CoercibleAsString());");
trace(coerces_param_into_Number(new CoercibleAsString()));

trace("//coerces_param_into_Number(new CoercibleAsValue());");
trace(coerces_param_into_Number(new CoercibleAsValue()));

trace("//coerces_param_into_Boolean(undefined);");
trace(coerces_param_into_Boolean(undefined));

trace("//coerces_param_into_Boolean(null);");
trace(coerces_param_into_Boolean(null));

trace("//coerces_param_into_Boolean(true);");
trace(coerces_param_into_Boolean(true));

trace("//coerces_param_into_Boolean(false);");
trace(coerces_param_into_Boolean(false));

trace("//coerces_param_into_Boolean(5.12);");
trace(coerces_param_into_Boolean(5.12));

trace("//coerces_param_into_Boolean(-6);");
trace(coerces_param_into_Boolean(-6));

trace("//coerces_param_into_Boolean(\"12.23\");");
trace(coerces_param_into_Boolean("12.23"));

trace("//coerces_param_into_Boolean(new CoercibleAsString());");
trace(coerces_param_into_Boolean(new CoercibleAsString()));

trace("//coerces_param_into_Boolean(new CoercibleAsValue());");
trace(coerces_param_into_Boolean(new CoercibleAsValue()));

trace("//coerces_param_into_String(undefined);");
trace(coerces_param_into_String(undefined));

trace("//coerces_param_into_String(null);");
trace(coerces_param_into_String(null));

trace("//coerces_param_into_String(true);");
trace(coerces_param_into_String(true));

trace("//coerces_param_into_String(false);");
trace(coerces_param_into_String(false));

trace("//coerces_param_into_String(5.12);");
trace(coerces_param_into_String(5.12));

trace("//coerces_param_into_String(-6);");
trace(coerces_param_into_String(-6));

trace("//coerces_param_into_String(\"12.23\");");
trace(coerces_param_into_String("12.23"));

trace("//coerces_param_into_String(new CoercibleAsString());");
trace(coerces_param_into_String(new CoercibleAsString()));

trace("//coerces_param_into_String(new CoercibleAsValue());");
trace(coerces_param_into_String(new CoercibleAsValue()));

trace("//coerces_param_into_Object(undefined);");
trace(coerces_param_into_Object(undefined));

trace("//coerces_param_into_Object(null);");
trace(coerces_param_into_Object(null));

trace("//coerces_param_into_Object(true);");
trace(coerces_param_into_Object(true));

trace("//coerces_param_into_Object(false);");
trace(coerces_param_into_Object(false));

trace("//coerces_param_into_Object(5.12);");
trace(coerces_param_into_Object(5.12));

trace("//coerces_param_into_Object(-6);");
trace(coerces_param_into_Object(-6));

trace("//coerces_param_into_Object(\"12.23\");");
trace(coerces_param_into_Object("12.23"));

trace("//coerces_param_into_Object(new CoercibleAsString());");
trace(coerces_param_into_Object(new CoercibleAsString()));

trace("//coerces_param_into_Object(new CoercibleAsValue());");
trace(coerces_param_into_Object(new CoercibleAsValue()));