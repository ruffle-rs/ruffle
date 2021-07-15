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

trace("/// (contents of a_bool.slice()...)");
trace_vector(a_bool.slice());

trace("/// (contents of a_bool.slice(1)...)");
trace_vector(a_bool.slice(1));

trace("/// (contents of a_bool.slice(0, 1)...)");
trace_vector(a_bool.slice(0, 1));

trace("/// (contents of a_bool.slice(15)...)");
trace_vector(a_bool.slice(15));

trace("/// (contents of a_bool.slice(0, 0)...)");
trace_vector(a_bool.slice(0, 0));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of b_bool.slice()...)");
trace_vector(b_bool.slice());

trace("/// (contents of b_bool.slice(-1)...)");
trace_vector(b_bool.slice(-1));

trace("/// (contents of b_bool.slice(-1, -4)...)");
trace_vector(b_bool.slice(-1, -4));

trace("/// (contents of b_bool.slice(-1, 2)...)");
trace_vector(b_bool.slice(-1, 2));

trace("/// (contents of b_bool.slice(-3, 1)...)");
trace_vector(b_bool.slice(-3, 1));

trace("/// (contents of b_bool.slice(1, -2)...)");
trace_vector(b_bool.slice(1, -2));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of c_bool.slice()...)");
trace_vector(c_bool.slice());

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

trace("/// var a_sliced = a_class.slice();");
var a_sliced = a_class.slice();

trace("/// (contents of a_sliced...)");
trace_vector(a_sliced);

trace("/// a0_class === a_sliced[0];");
trace(a0_class === a_sliced[0]);

trace("/// a1_class === a_sliced[1];");
trace(a1_class === a_sliced[1]);

trace("/// (contents of a_class.slice(1)...)");
trace_vector(a_class.slice(1));

trace("/// (contents of a_class.slice(0, 1)...)");
trace_vector(a_class.slice(0, 1));

trace("/// (contents of a_class.slice(0, 3)...)");
trace_vector(a_class.slice(0, 3));

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of b_class.slice(-1, -16777216)...)");
trace_vector(b_class.slice(-1, -16777216));

trace("/// (contents of b_class.slice(-1)...)");
trace_vector(b_class.slice(-1));

trace("/// (contents of b_class.slice(-2)...)");
trace_vector(b_class.slice(-2));

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

trace("/// (contents of a_int.slice())...");
trace_vector_int(a_int.slice());

trace("/// (contents of a_int.slice(1))...");
trace_vector_int(a_int.slice(1));

trace("/// (contents of a_int.slice(0, 1))...");
trace_vector_int(a_int.slice(0, 1));

trace("/// (contents of a_int.slice(0, 2))...");
trace_vector_int(a_int.slice(0, 2));

trace("/// (contents of a_int.slice(0, 0))...");
trace_vector_int(a_int.slice(0, 0));

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// (contents of b_int.slice(-1))...");
trace_vector_int(b_int.slice(-1));

trace("/// (contents of b_int.slice(0, -3))...");
trace_vector_int(b_int.slice(0, -3));

trace("/// (contents of b_int.slice(0, -1))...");
trace_vector_int(b_int.slice(0, -1));

trace("/// (contents of b_int.slice(-1, -1))...");
trace_vector_int(b_int.slice(-1, -1));

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

trace("/// (contents of a_number.slice()...)");
trace_vector_number(a_number.slice());

trace("/// (contents of a_number.slice(2)...)");
trace_vector_number(a_number.slice(2));

trace("/// (contents of a_number.slice(0, 2)...)");
trace_vector_number(a_number.slice(0, 2));

trace("/// (contents of a_number.slice(16, 32)...)");
trace_vector_number(a_number.slice(16, 32));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// (contents of b_number.slice(-1)...)");
trace_vector_number(b_number.slice(-1));

trace("/// (contents of b_number.slice(-2, 4)...)");
trace_vector_number(b_number.slice(-2, 4));

trace("/// (contents of b_number.slice(2, -4)...)");
trace_vector_number(b_number.slice(2, -4));

trace("/// (contents of b_number.slice(-16, 0)...)");
trace_vector_number(b_number.slice(-16, 0));

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

trace("/// (contents of a_string.slice()...)");
trace_vector_string(a_string.slice());

trace("/// (contents of a_string.slice(0)...)");
trace_vector_string(a_string.slice(0));

trace("/// (contents of a_string.slice(2)...)");
trace_vector_string(a_string.slice(2));

trace("/// (contents of a_string.slice(4, 2)...)");
trace_vector_string(a_string.slice(4, 2));

trace("/// (contents of a_string.slice(0, 16)...)");
trace_vector_string(a_string.slice(0, 16));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of b_string.slice(-1)...)");
trace_vector_string(b_string.slice(-1));

trace("/// (contents of b_string.slice(-3, -1)...)");
trace_vector_string(b_string.slice(-3, -1));

trace("/// (contents of b_string.slice(-16, 32)...)");
trace_vector_string(b_string.slice(-16, 32));

trace("/// (contents of b_string.slice(1, -32)...)");
trace_vector_string(b_string.slice(1, -32));

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

trace("/// (contents of a_uint.slice()...)");
trace_vector_uint(a_uint.slice());

trace("/// (contents of a_uint.slice(0, 2)...)");
trace_vector_uint(a_uint.slice(0, 2));

trace("/// (contents of a_uint.slice(15, 0)...)");
trace_vector_uint(a_uint.slice(15, 0));

trace("/// (contents of a_uint.slice(0, 0)...)");
trace_vector_uint(a_uint.slice(0, 0));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// (contents of b_uint.slice()...)");
trace_vector_uint(b_uint.slice());

trace("/// (contents of b_uint.slice(-1)...)");
trace_vector_uint(b_uint.slice(-1));

trace("/// (contents of b_uint.slice(-2, -1)...)");
trace_vector_uint(b_uint.slice(-2, -1));

trace("/// (contents of b_uint.slice(0, -16)...)");
trace_vector_uint(b_uint.slice(0, -16));

trace("/// (contents of b_uint.slice(-16, 0)...)");
trace_vector_uint(b_uint.slice(-16, 0));

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

trace("/// (contents of a_vector.slice()...)");
trace_vector_vector(a_vector.slice());

trace("/// (contents of a_vector.slice(1)...)");
trace_vector_vector(a_vector.slice(1));

trace("/// (contents of a_vector.slice(0, 3)...)");
trace_vector_vector(a_vector.slice(0, 3));

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// (contents of b_vector.slice()...)");
trace_vector_vector(b_vector.slice());

trace("/// (contents of b_vector.slice(-1)...)");
trace_vector_vector(b_vector.slice(-1));

trace("/// (contents of b_vector.slice(-16)...)");
trace_vector_vector(b_vector.slice(-16));

trace("/// (contents of b_vector.slice(0, -13)...)");
trace_vector_vector(b_vector.slice(0, -13));

trace("/// (contents of b_vector.slice(-16, 0)...)");
trace_vector_vector(b_vector.slice(-16, 0));

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);