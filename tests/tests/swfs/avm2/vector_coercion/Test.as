package {
	public class Test {
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[1,2,3,4];");
var a_bool:Vector.<Boolean> = new <Boolean>[1,2,3,4];

trace("/// a_bool[0] = 1;");
a_bool[0] = 1;

trace("/// a_bool[1] = NaN;");
a_bool[1] = NaN;

trace("/// a_bool[2] = \"false\";");
a_bool[2] = "false";

trace("/// a_bool[3] = true;");
a_bool[3] = true;

trace(a_bool[0]);
trace(a_bool[1]);
trace(a_bool[2]);
trace(a_bool[3]);

function LegacyClass() {
	
}

function LegacySubclass() {
	
}

LegacySubclass.prototype = new LegacyClass();

trace("/// var a_legacy: Vector.<Object> = new <Object>[];");
var a_legacy:Vector.<Object> = new <Object>[];

trace("/// a_legacy.length = 2;");
a_legacy.length = 2;

trace(a_legacy[0]);
trace(a_legacy[1]);

trace("/// a_legacy[0] = new LegacyClass();");
a_legacy[0] = new LegacyClass();

trace("/// a_legacy[1] = new LegacySubclass();");
a_legacy[1] = new LegacySubclass();

trace(a_legacy[0]);
trace(a_legacy[1]);

class Superclass {
	
}

class Subclass extends Superclass {
	
}

trace("/// var a_class: Vector.<Superclass> = new <Superclass>[];");
var a_class:Vector.<Superclass> = new <Superclass>[];

trace("/// a_class.length = 2;");
a_class.length = 2;

trace(a_class[0]);
trace(a_class[1]);

trace("/// a_class[0] = new Superclass();");
a_class[0] = new Superclass();

trace("/// a_class[1] = new Subclass();");
a_class[1] = new Subclass();

trace(a_class[0]);
trace(a_class[1]);

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// a_int[0] = \"5\";");
a_int[0] = "5";

trace("/// a_int[1] = \"not a number\";");
a_int[1] = "not a number";

trace(a_int[0]);
trace(a_int[1]);

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// a_number[0] = \"5\";");
a_number[0] = "5";

trace("/// a_number[1] = \"NaN\";");
a_number[1] = "NaN";

trace("/// a_number[2] = -5;");
a_number[2] = -5;

trace("/// a_number[3] = true;");
a_number[3] = true;

trace(a_number[0]);
trace(a_number[1]);
trace(a_number[2]);
trace(a_number[3]);

trace("/// var a_string: Vector.<String> = new <String>[1,2,3,4];");
var a_string:Vector.<String> = new <String>[1,2,3,4];

trace("/// a_string[0] = 5;");
a_string[0] = 5;

trace("/// a_string[1] = NaN;");
a_string[1] = NaN;

trace("/// a_string[2] = \"actually imma string\";");
a_string[2] = "actually imma string";

trace("/// a_string[3] = true;");
a_string[3] = true;

trace(a_string[0]);
trace(a_string[1]);
trace(a_string[2]);
trace(a_string[3]);

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2,3,4];");
var a_uint:Vector.<uint> = new <uint>[1,2,3,4];

trace("/// a_uint[0] = \"5\";");
a_uint[0] = "5";

trace("/// a_uint[1] = \"not a number\";");
a_uint[1] = "not a number";

trace("/// a_uint[2] = -5;");
a_uint[2] = -5;

trace("/// a_uint[3] = false;");
a_uint[3] = false;

trace(a_uint[0]);
trace(a_uint[1]);
trace(a_uint[2]);
trace(a_uint[3]);

function trace_vector(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_vector: Vector.<int> = new <int>[1,2];");
var a_vector:Vector.<int> = new <int>[1,2];

trace("/// var b_vector: Vector.<int> = new <int>[5,16];");
var b_vector:Vector.<int> = new <int>[5,16];

trace("/// var c_vector: Vector.<Vector.<int>> = new <Vector.<int>>[];");
var c_vector:Vector.<Vector.<int>> = new <Vector.<int>>[];

trace("/// c_vector[0] = a_vector;");
c_vector[0] = a_vector;

trace("/// c_vector[1] = b_vector;");
c_vector[1] = b_vector;

trace("/// (contents of c_vector...)");
trace_vector(c_vector);