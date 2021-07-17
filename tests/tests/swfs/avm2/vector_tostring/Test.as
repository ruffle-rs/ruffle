package {
	public class Test {
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[true, false];");
var a_bool:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b_bool:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// var c_bool: Vector.<Boolean> = new <Boolean>[];");
var c_bool:Vector.<Boolean> = new <Boolean>[];

trace("/// a_bool.toString()");
trace(a_bool.toString());

trace("/// a_bool.toLocaleString()");
trace(a_bool.toLocaleString());

trace("/// b_bool.toString()");
trace(b_bool.toString());

trace("/// b_bool.toLocaleString()");
trace(b_bool.toLocaleString());

trace("/// c_bool.toString()");
trace(c_bool.toString());

trace("/// c_bool.toLocaleString()");
trace(c_bool.toLocaleString());

class Superclass {
	
}

class Subclass extends Superclass {
	
}

trace("/// var a0_class = new Superclass();");
var a0_class = new Superclass();

trace("/// var a1_class = new Subclass();");
var a1_class = new Subclass();

trace("/// var a_class: Vector.<Superclass> = new <Superclass>[a0_class, a1_class];");
var a_class:Vector.<Superclass> = new <Superclass>[a0_class, a1_class];

trace("/// var b_class: Vector.<Subclass> = new <Subclass>[];");
var b_class:Vector.<Subclass> = new <Subclass>[];

trace("/// b_class.length = 1;");
b_class.length = 1;

trace("/// b_class[0] = new Subclass();");
b_class[0] = new Subclass();

trace("/// a_class.toString()");
trace(a_class.toString());

trace("/// a_class.toLocaleString()");
trace(a_class.toLocaleString());

trace("/// b_class.toString()");
trace(b_class.toString());

trace("/// b_class.toLocaleString()");
trace(b_class.toLocaleString());

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.toString()");
trace(a_int.toString());

trace("/// a_int.toLocaleString()");
trace(a_int.toLocaleString());

trace("/// b_int.toString()");
trace(b_int.toString());

trace("/// b_int.toLocaleString()");
trace(b_int.toLocaleString());

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.toString()");
trace(a_number.toString());

trace("/// a_number.toLocaleString()");
trace(a_number.toLocaleString());

trace("/// b_number.toString()");
trace(b_number.toString());

trace("/// b_number.toLocaleString()");
trace(b_number.toLocaleString());

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.toString()");
trace(a_string.toString());

trace("/// a_string.toLocaleString()");
trace(a_string.toLocaleString());

trace("/// b_string.toString()");
trace(b_string.toString());

trace("/// b_string.toLocaleString()");
trace(b_string.toLocaleString());

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.toString()");
trace(a_uint.toString());

trace("/// a_uint.toLocaleString()");
trace(a_uint.toLocaleString());

trace("/// b_uint.toString()");
trace(b_uint.toString());

trace("/// b_uint.toLocaleString()");
trace(b_uint.toLocaleString());

function trace_vector_vector(v) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		if (v[i] is Vector.<int>) {
			trace("/// (contents of index", i, ")");
			trace_vector_vector(v[i]);
		} else {
			trace(v[i]);
		}
	}
}

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// a_vector.toString()");
trace(a_vector.toString());

trace("/// a_vector.toLocaleString()");
trace(a_vector.toLocaleString());

trace("/// b_vector.toString()");
trace(b_vector.toString());

trace("/// b_vector.toLocaleString()");
trace(b_vector.toLocaleString());