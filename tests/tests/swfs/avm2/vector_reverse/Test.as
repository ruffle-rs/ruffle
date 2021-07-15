package {
	public class Test {
	}
}

function trace_vector(v: Vector.<*>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[true, false];");
var a_bool:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b_bool:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// var c_bool: Vector.<Boolean> = new <Boolean>[];");
var c_bool:Vector.<Boolean> = new <Boolean>[];

trace("/// a_bool.reverse();");
trace(a_bool.reverse());

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// a_bool.reverse();");
trace(a_bool.reverse());

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// b_bool.reverse();");
trace(b_bool.reverse());

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// b_bool.reverse();");
trace(b_bool.reverse());

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// c_bool.reverse();");
trace(c_bool.reverse());

trace("/// (contents of c_bool...)");
trace_vector(c_bool);

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

trace("/// a_class.reverse();");
trace(a_class.reverse());

trace("/// a1_class === a_class[0];");
trace(a1_class === a_class[0]);

trace("/// a0_class === a_class[1];");
trace(a0_class === a_class[1]);

trace("/// b_class.reverse();");
trace(b_class.reverse());

trace("/// (contents of b_class...)");
trace_vector(b_class);

function trace_vector_int(v: Vector.<int>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.reverse();");
trace(a_int.reverse());

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// a_int.reverse();");
trace(a_int.reverse());

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// b_int.reverse();");
trace(b_int.reverse());

trace("/// (contents of b_int)...");
trace_vector_int(b_int);

trace("/// b_int.reverse();");
trace(b_int.reverse());

trace("/// (contents of b_int)...");
trace_vector_int(b_int);

function trace_vector_number(v: Vector.<Number>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.reverse();");
trace(a_number.reverse());

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// a_number.reverse();");
trace(a_number.reverse());

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// b_number.reverse();");
trace(b_number.reverse());

trace("/// (contents of b_number...)");
trace_vector_number(b_number);

trace("/// b_number.reverse();");
trace(b_number.reverse());

trace("/// (contents of b_number...)");
trace_vector_number(b_number);

function trace_vector_string(v: Vector.<String>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.reverse();");
trace(a_string.reverse());

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// a_string.reverse();");
trace(a_string.reverse());

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// b_string.reverse();");
trace(b_string.reverse());

trace("/// (contents of b_string...)");
trace_vector_string(b_string);

trace("/// b_string.reverse();");
trace(b_string.reverse());

trace("/// (contents of b_string...)");
trace_vector_string(b_string);

function trace_vector_uint(v: Vector.<uint>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.reverse();");
trace(a_uint.reverse());

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// a_uint.reverse();");
trace(a_uint.reverse());

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// b_uint.reverse();");
trace(b_uint.reverse());

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// b_uint.reverse();");
trace(b_uint.reverse());

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

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

trace("/// a_vector.reverse();");
trace(a_vector.reverse());

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// a_vector.reverse();");
trace(a_vector.reverse());

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// b_vector.reverse();");
trace(b_vector.reverse());

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);

trace("/// b_vector.reverse();");
trace(b_vector.reverse());

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);

trace("/// (contents of b_vector.reverse()...)");
trace_vector_vector(b_vector.reverse());

trace("/// b_vector === b_vector.reverse()");
trace(b_vector === b_vector.reverse());