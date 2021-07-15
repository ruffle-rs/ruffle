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

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[true, true];");
var b_bool:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a_bool.map(function(v) { return v; });");
trace_vector(a_bool.map(function (v) { return v; }));

trace("/// a_bool.map(function(v) { return !v; });");
trace_vector(a_bool.map(function (v) { return !v; }));

trace("/// b_bool.map(function(v) { return v; });");
trace_vector(b_bool.map(function (v) { return v; }));

trace("/// b_bool.map(function(v) { return !v; });");
trace_vector(b_bool.map(function (v) { return !v; }));

class Superclass {
	
}

class Subclass extends Superclass {
	
}

trace("/// var a_class: Vector.<Superclass> = new <Superclass>[];");
var a_class:Vector.<Superclass> = new <Superclass>[];

trace("/// a_class.length = 2;");
a_class.length = 2;

trace("/// a_class[0] = new Superclass();");
a_class[0] = new Superclass();

trace("/// a_class[1] = new Subclass();");
a_class[1] = new Subclass();

trace("/// var b_class: Vector.<Subclass> = new <Subclass>[];");
var b_class:Vector.<Subclass> = new <Subclass>[];

trace("/// b_class.length = 1;");
b_class.length = 1;

trace("/// b_class[0] = new Subclass();");
b_class[0] = new Subclass();

trace("/// a_class.map(function (v) { return new Superclass(); }));");
trace(a_class.map(function (v) { return new Superclass(); }));

trace("/// a_class.map(function (v) { return new Superclass(); }));");
trace(a_class.map(function (v) { return new Superclass(); }));

trace("/// b_class.map(function (v) { return new Subclass(); }));");
trace(b_class.map(function (v) { return new Subclass(); }));

trace("/// b_class.map(function (v) { return new Subclass(); }));");
trace(b_class.map(function (v) { return new Subclass(); }));

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.map(function (v) { return v * 2; }));");
trace(a_int.map(function (v) { return v * 2; }));

trace("/// a_int.map(function (v) { return v * -.5; }));");
trace(a_int.map(function (v) { return v * -.5; }));

trace("/// b_int.map(function (v) { return v * 3; }));");
trace(b_int.map(function (v) { return v * 3; }));

trace("/// b_int.map(function (v) { return v * -6; }));");
trace(b_int.map(function (v) { return v * -6; }));

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.map(function (v) { return \"6\"; }));");
trace(a_number.map(function (v) { return "6"; }));

trace("/// a_number.map(function (v) { return v * 6; }));");
trace(a_number.map(function (v) { return v * 6; }));

trace("/// b_number.map(function (v) { return v * -1; }));");
trace(b_number.map(function (v) { return v * -1; }));

trace("/// b_number.map(function (v) { return v * -2; }));");
trace(b_number.map(function (v) { return v * -2; }));

trace("/// b_number.map(function (v) { return v * -6; }));");
trace(b_number.map(function (v) { return v * -6; }));

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.map(function (v) { return v.length; }));");
trace(a_string.map(function (v) { return v.length; }));

trace("/// a_string.map(function (v) { return v + \" and\"; }));");
trace(a_string.map(function (v) { return v + " and"; }));

trace("/// b_string.map(function (v) { return v.length; }));");
trace(b_string.map(function (v) { return v.length; }));

trace("/// b_string.map(function (v) { return v + \"6\"; }));");
trace(b_string.map(function (v) { return v + "6"; }));

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.map(function (v) { return v * -1; }));");
trace(a_uint.map(function (v) { return v * -1; }));

trace("/// a_uint.map(function (v) { return v * -0.5; }));");
trace(a_uint.map(function (v) { return v * -0.5; }));

trace("/// b_uint.map(function (v) { return v * 3; }));");
trace(b_uint.map(function (v) { return v * 3; }));

trace("/// b_uint.map(function (v) { return v * -6; }));");
trace(b_uint.map(function (v) { return v * -6; }));

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// a_vector.map(function (v) { return v.map(function (v) { return v * -6; }); });");
trace(a_vector.map(function (v) { return v.map(function (v) { return v * -6; }); }));

trace("/// a_vector.map(function (v) { return v.map(function (v) { return v * 2; }); });");
trace(a_vector.map(function (v) { return v.map(function (v) { return v * 2; }); }));

trace("/// b_vector.map(function (v) { return v.map(function (v) { return v * 4; }); });");
trace(b_vector.map(function (v) { return v.map(function (v) { return v * 4; }); }));

trace("/// b_vector.map(function (v) { return v.map(function (v) { return v * 6.5; }); });");
trace(b_vector.map(function (v) { return v.map(function (v) { return v * 6.5; }); }));