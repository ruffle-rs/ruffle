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

trace("/// (contents of a_bool.splice(1, 0, false)...)");
trace_vector(a_bool.splice(1, 0, false));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of a_bool.splice(0, 1, false, false, false)...)");
trace_vector(a_bool.splice(0, 1, false, false, false));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of a_bool.splice(15, true)...)");
trace_vector(a_bool.splice(15, true));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of a_bool.splice(0, 0)...)");
trace_vector(a_bool.splice(0, 0));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of b_bool.splice(-1, 1)...)");
trace_vector(b_bool.splice(-1, 1));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of b_bool.splice(-1, -4)...)");
trace_vector(b_bool.splice(-1, -4));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of b_bool.splice(-1, 2)...)");
trace_vector(b_bool.splice(-1, 2));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of b_bool.splice(-3, 1)...)");
trace_vector(b_bool.splice(-3, 1));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of b_bool.splice(1, -2)...)");
trace_vector(b_bool.splice(1, -2));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of c_bool.splice(0, 4000)...)");
trace_vector(c_bool.splice(0, 4000));

trace("/// (contents of c_bool...)");
trace_vector(c_bool);

trace("/// (contents of c_bool.splice(0, 0, false, true)...)");
trace_vector(c_bool.splice(0, 0, false, true));

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

trace("/// var a_sliced = a_class.splice(0, 2);");
var a_sliced = a_class.splice(0, 2);

trace("/// (contents of a_sliced...)");
trace_vector(a_sliced);

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// a0_class === a_sliced[0];");
trace(a0_class === a_sliced[0]);

trace("/// a1_class === a_sliced[1];");
trace(a1_class === a_sliced[1]);

trace("/// (contents of a_class.splice(0, 0, a0_class, a1_class)...)");
trace_vector(a_class.splice(0, 0, a0_class, a1_class));

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of a_class.splice(0, 1)...)");
trace_vector(a_class.splice(0, 1));

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of a_class.splice(0, 0, a0_class, a1_class)...)");
trace_vector(a_class.splice(0, 0, a0_class, a1_class));

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of a_class.splice(0, 3)...)");
trace_vector(a_class.splice(0, 3));

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of b_class.splice(-1, -16777216)...)");
trace_vector(b_class.splice(-1, -16777216));

trace("/// (contents of b_class...)");
trace_vector(b_class);

trace("/// (contents of b_class.splice(-1, 0)...)");
trace_vector(b_class.splice(-1, 0));

trace("/// (contents of b_class...)");
trace_vector(b_class);

trace("/// (contents of b_class.splice(-2, 555)...)");
trace_vector(b_class.splice(-2, 555));

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

trace("/// (contents of a_int.splice(0, 0))...");
trace_vector_int(a_int.splice(0, 0));

trace("/// (contents of a_int...)");
trace_vector_int(a_int);

trace("/// (contents of a_int.splice(1, 1, 3, 4))...");
trace_vector_int(a_int.splice(1, 1, 3, 4));

trace("/// (contents of a_int...)");
trace_vector_int(a_int);

trace("/// (contents of a_int.splice(0, 500, 5, 6))...");
trace_vector_int(a_int.splice(0, 500, 5, 6));

trace("/// (contents of a_int...)");
trace_vector_int(a_int);

trace("/// (contents of a_int.splice(0, 2))...");
trace_vector_int(a_int.splice(0, 2));

trace("/// (contents of a_int...)");
trace_vector_int(a_int);

trace("/// (contents of a_int.splice(0, 0))...");
trace_vector_int(a_int.splice(0, 0));

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// (contents of b_int.splice(-1, -5))...");
trace_vector_int(b_int.splice(-1, -5));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(0, -3, 18, 20))...");
trace_vector_int(b_int.splice(0, -3, 18, 20));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(1, -3, 21, 22))...");
trace_vector_int(b_int.splice(1, -3, 21, 22));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(2, -3))...");
trace_vector_int(b_int.splice(2, -3));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(3, -3))...");
trace_vector_int(b_int.splice(3, -3));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(4, -3))...");
trace_vector_int(b_int.splice(4, -3));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(-2, 3, 23, 24))...");
trace_vector_int(b_int.splice(-2, 3, 23, 24));

trace("/// (contents of b_int...)");
trace_vector_int(b_int);

trace("/// (contents of b_int.splice(-1, 2))...");
trace_vector_int(b_int.splice(-1, 2));

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

trace("/// (contents of a_number.splice(0, 5, 1, 2, 3, 4)...)");
trace_vector_number(a_number.splice(0, 5, 1, 2, 3, 4));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// (contents of a_number.splice(2, 1)...)");
trace_vector_number(a_number.splice(2, 1));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// (contents of a_number.splice(0, 2)...)");
trace_vector_number(a_number.splice(0, 2));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// (contents of a_number.splice(16, 32)...)");
trace_vector_number(a_number.splice(16, 32));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// (contents of b_number.splice(-1,6,7,8,9)...)");
trace_vector_number(b_number.splice(-1,6,7,8,9));

trace("/// (contents of b_number...)");
trace_vector_number(b_number);

trace("/// (contents of b_number.splice(-2, 4,10,11,12,13)...)");
trace_vector_number(b_number.splice(-2, 4,10,11,12,13));

trace("/// (contents of b_number...)");
trace_vector_number(b_number);

trace("/// (contents of b_number.splice(2, -4,14,15,16,17)...)");
trace_vector_number(b_number.splice(2, -4,14,15,16,17));

trace("/// (contents of b_number...)");
trace_vector_number(b_number);

trace("/// (contents of b_number.splice(-16, 0,18)...)");
trace_vector_number(b_number.splice(-16, 0,18));

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

trace("/// (contents of a_string.splice(2, 1)...)");
trace_vector_string(a_string.splice(2, 1));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of a_string.splice(0, 0, \"g\")...)");
trace_vector_string(a_string.splice(0, 0, "g"));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of a_string.splice(2, 2)...)");
trace_vector_string(a_string.splice(2, 2));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of a_string.splice(4, 2, \"h\", \"i\", \"j\")...)");
trace_vector_string(a_string.splice(4, 2, "h", "i", "j"));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of a_string.splice(0, 16)...)");
trace_vector_string(a_string.splice(0, 16));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// (contents of b_string.splice(-1, 2)...)");
trace_vector_string(b_string.splice(-1, 2));

trace("/// (contents of b_string...)");
trace_vector_string(b_string);

trace("/// (contents of b_string.splice(-3, -1, 16, \"32\")...)");
trace_vector_string(b_string.splice(-3, -1, 16, "32"));

trace("/// (contents of b_string...)");
trace_vector_string(b_string);

trace("/// (contents of b_string.splice(-16, 32)...)");
trace_vector_string(b_string.splice(-16, 32));

trace("/// (contents of b_string...)");
trace_vector_string(b_string);

trace("/// (contents of b_string.splice(1, -32)...)");
trace_vector_string(b_string.splice(1, -32));

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

trace("/// (contents of a_uint.splice(0, 0)...)");
trace_vector_uint(a_uint.splice(0, 0));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// (contents of a_uint.splice(0, 2)...)");
trace_vector_uint(a_uint.splice(0, 2));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// (contents of a_uint.splice(15, 0, 3, 4)...)");
trace_vector_uint(a_uint.splice(15, 0, 3, 4));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// (contents of a_uint.splice(0, 0, 5)...)");
trace_vector_uint(a_uint.splice(0, 0, 5));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// (contents of b_uint.splice(-5, 0, 17)...)");
trace_vector_uint(b_uint.splice(-5, 0, 17));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-1, -2, -23)...)");
trace_vector_uint(b_uint.splice(-1, -2, -23));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-2, -1, \"55\")...)");
trace_vector_uint(b_uint.splice(-2, -1, "55"));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(0, -16, false, true, 56)...)");
trace_vector_uint(b_uint.splice(0, -16, false, true, 56));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, 0, 99)...)");
trace_vector_uint(b_uint.splice(-16, 0, 99));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, 1, 98)...)");
trace_vector_uint(b_uint.splice(-16, 1, 98));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, 2, 97)...)");
trace_vector_uint(b_uint.splice(-16, 2, 97));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -1, 96)...)");
trace_vector_uint(b_uint.splice(-16, 1, 96));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -2, 95)...)");
trace_vector_uint(b_uint.splice(-16, 2, 95));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -16, 94)...)");
trace_vector_uint(b_uint.splice(-16, -16, 94));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -15, 93)...)");
trace_vector_uint(b_uint.splice(-16, -15, 93));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -8, 92)...)");
trace_vector_uint(b_uint.splice(-16, -8, 92));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -7, 91)...)");
trace_vector_uint(b_uint.splice(-16, -7, 91));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -6, 90)...)");
trace_vector_uint(b_uint.splice(-16, -6, 90));

trace("/// (contents of b_uint...)");
trace_vector_uint(b_uint);

trace("/// (contents of b_uint.splice(-16, -5, 89)...)");
trace_vector_uint(b_uint.splice(-16, -5, 89));

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

trace("/// (contents of a_vector.splice(0, 0)...)");
trace_vector_vector(a_vector.splice(0, 0));

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// (contents of a_vector.splice(1, 1)...)");
trace_vector_vector(a_vector.splice(1, 1));

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// (contents of a_vector.splice(0, 3, new <int>[5,6])...)");
trace_vector_vector(a_vector.splice(0, 3, new <int>[5,6]));

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// (contents of b_vector.splice(-1, -1, new <int>[])...)");
trace_vector_vector(b_vector.splice(-1, -1, new <int>[]));

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);

trace("/// (contents of b_vector.splice(0, -13, new <int>[86,13])...)");
trace_vector_vector(b_vector.splice(0, -13, new <int>[86,13]));

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);

trace("/// (contents of b_vector.splice(-16, 0, new <int>[99])...)");
trace_vector_vector(b_vector.splice(-16, 0, new <int>[99]));

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);