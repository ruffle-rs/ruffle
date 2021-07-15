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

trace("/// a_bool.shift();");
trace(a_bool.shift());

trace("/// a_bool.shift();");
trace(a_bool.shift());

trace("/// a_bool.shift();");
trace(a_bool.shift());

trace("/// a_bool.unshift(0, \"true\", -1, 3.5, \"false\", false, true);");
trace(a_bool.unshift(0, "true", -1, 3.5, "false", false, true));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// b_bool.unshift(0, \"true\", -1, 3.5, \"false\", false, true);");
trace(b_bool.unshift(0, "true", -1, 3.5, "false", false, true));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// c_bool.unshift();");
trace(c_bool.unshift());

trace("/// (contents of c_bool...)");
trace_vector(c_bool);

trace("/// c_bool.unshift(false);");
trace(c_bool.unshift(false));

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

trace("/// a_class.shift();");
trace(a_class.shift());

trace("/// a_class.shift();");
trace(a_class.shift());

trace("/// a_class.shift();");
trace(a_class.shift());

trace("/// a_class.unshift(a1_class, a0_class, a0_class, a1_class);");
trace(a_class.unshift(a1_class, a0_class, a0_class, a1_class));

trace("/// a_class[0] === a_class[3];");
trace(a_class[0] === a_class[3]);

trace("/// a_class[1] === a_class[2];");
trace(a_class[1] === a_class[2]);

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// b_class.unshift(new Subclass());");
trace(b_class.unshift(new Subclass()));

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

trace("/// a_int.shift();");
trace(a_int.shift());

trace("/// a_int.unshift(5);");
trace(a_int.unshift(5));

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// a_int.shift();");
trace(a_int.shift());

trace("/// a_int.shift();");
trace(a_int.shift());

trace("/// a_int.unshift(-15, 32, true, false, \"63\");");
trace(a_int.unshift(-15, 32, true, false, "63"));

trace("/// (contents of a_int)...");
trace_vector_int(a_int);

trace("/// b_int.shift();");
trace(b_int.shift());

trace("/// b_int.shift();");
trace(b_int.shift());

trace("/// b_int.shift();");
trace(b_int.shift());

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

trace("/// a_number.shift();");
trace(a_number.shift());

trace("/// a_number.shift();");
trace(a_number.shift());

trace("/// a_number.unshift(-16, 3.2, 5, \"test\", true, false);");
trace(a_number.unshift(-16, 3.2, 5, "test", true, false));

trace("/// (contents of a_number...)");
trace_vector_number(a_number);

trace("/// b_number.shift();");
trace(b_number.shift());

trace("/// b_number.shift();");
trace(b_number.shift());

trace("/// b_number.unshift(NaN, \"NaN\", 0);");
trace(b_number.unshift(NaN, "NaN", 0));

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

trace("/// a_string.shift();");
trace(a_string.shift());

trace("/// a_string.shift();");
trace(a_string.shift());

trace("/// a_string.shift();");
trace(a_string.shift());

trace("/// a_string.shift();");
trace(a_string.shift());

trace("/// a_string.shift();");
trace(a_string.shift());

trace("/// a_string.unshift(123, {}, \"abc\", true, false);");
trace(a_string.unshift(123, {}, "abc", true, false));

trace("/// (contents of a_string...)");
trace_vector_string(a_string);

trace("/// b_string.shift();");
trace(b_string.shift());

trace("/// b_string.unshift(NaN, -83.5);");
trace(b_string.unshift(NaN, -83.5));

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

trace("/// a_uint.shift();");
trace(a_uint.shift());

trace("/// a_uint.shift();");
trace(a_uint.shift());

trace("/// a_uint.shift();");
trace(a_uint.shift());

trace("/// a_uint.unshift(0, -1, -2.5, \"16\", \"NaN\");");
trace(a_uint.unshift(0, -1, -2.5, "16", "NaN"));

trace("/// (contents of a_uint...)");
trace_vector_uint(a_uint);

trace("/// b_uint.shift();");
trace(b_uint.shift());

trace("/// b_uint.unshift(true, 15.23, \"true\");");
trace(b_uint.unshift(true, 15.23, "true"));

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

trace("/// (contents of a_vector.shift()...)");
trace_vector_vector(a_vector.shift());

trace("/// (contents of a_vector.shift()...)");
trace_vector_vector(a_vector.shift());

trace("/// a_vector.shift();");
trace(a_vector.shift());

trace("/// a_vector.unshift(new <int>[15,9], new <int>[-1,-94], new <int>[2], new <int>[16]);");
trace(a_vector.unshift(new <int>[15,9], new <int>[-1,-94], new <int>[2], new <int>[16]));

trace("/// (contents of a_vector...)");
trace_vector_vector(a_vector);

trace("/// (contents of b_vector.shift()...)");
trace_vector_vector(b_vector.shift());

trace("/// b_vector.unshift(new <int>[-1,-94]);");
trace(b_vector.unshift(new <int>[-16,-4]));

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);

trace("/// b_vector.length = 6;");
trace(b_vector.length = 6);

trace("/// b_vector.shift()");
trace(b_vector.shift());

trace("/// (contents of b_vector...)");
trace_vector_vector(b_vector);