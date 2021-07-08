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

trace("/// a_bool.filter(function(v) { return v; });");
trace_vector(a_bool.filter(function (v) { return v; }));

trace("/// a_bool.filter(function(v) { return !v; });");
trace_vector(a_bool.filter(function (v) { return !v; }));

trace("/// b_bool.filter(function(v) { return v; });");
trace_vector(b_bool.filter(function (v) { return v; }));

trace("/// b_bool.filter(function(v) { return !v; });");
trace_vector(b_bool.filter(function (v) { return !v; }));

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

trace("/// a_class.filter(function (v) { return v is Subclass; }));");
trace(a_class.filter(function (v) { return v is Subclass; }));

trace("/// a_class.filter(function (v) { return v is Superclass; }));");
trace(a_class.filter(function (v) { return v is Superclass; }));

trace("/// b_class.filter(function (v) { return v is Subclass; }));");
trace(b_class.filter(function (v) { return v is Subclass; }));

trace("/// b_class.filter(function (v) { return v is Superclass; }));");
trace(b_class.filter(function (v) { return v is Superclass; }));

interface Interface {
	
}

class Implementer implements Interface {
	
}

trace("/// var a_iface: Vector.<Interface> = new <Interface>[];");
var a_iface:Vector.<Interface> = new <Interface>[];

trace("/// a_iface.length = 1;");
a_iface.length = 1;

trace("/// a_iface[0] = new Implementer();");
a_iface[0] = new Implementer();

trace("/// var b_iface: Vector.<Implementer> = new <Implementer>[];");
var b_iface:Vector.<Implementer> = new <Implementer>[];

trace("/// b_iface.length = 2;");
b_iface.length = 2;

trace("/// b_iface[0] = new Implementer();");
b_iface[0] = new Implementer();

trace("/// b_iface[1] = new Implementer();");
b_iface[1] = new Implementer();

trace("/// a_iface.filter(function (v) { return v is Implementer; }));");
trace(a_iface.filter(function (v) { return v is Implementer; }));

trace("/// a_iface.filter(function (v) { return v is Interface; }));");
trace(a_iface.filter(function (v) { return v is Interface; }));

trace("/// b_iface.filter(function (v) { return v is Implementer; }));");
trace(b_iface.filter(function (v) { return v is Implementer; }));

trace("/// b_iface.filter(function (v) { return v is Interface; }));");
trace(b_iface.filter(function (v) { return v is Interface; }));

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.filter(function (v) { return v > 0; }));");
trace(a_int.filter(function (v) { return v > 0; }));

trace("/// a_int.filter(function (v) { return v > 2; }));");
trace(a_int.filter(function (v) { return v > 2; }));

trace("/// b_int.filter(function (v) { return v > 4; }));");
trace(b_int.filter(function (v) { return v > 4; }));

trace("/// b_int.filter(function (v) { return v > 10; }));");
trace(b_int.filter(function (v) { return v > 10; }));

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.filter(function (v) { return v > 0; }));");
trace(a_number.filter(function (v) { return v > 0; }));

trace("/// a_number.filter(function (v) { return v > 2; }));");
trace(a_number.filter(function (v) { return v > 2; }));

trace("/// b_number.filter(function (v) { return v > 4; }));");
trace(b_number.filter(function (v) { return v > 4; }));

trace("/// b_number.filter(function (v) { return v > 10; }));");
trace(b_number.filter(function (v) { return v > 10; }));

trace("/// b_number.filter(function (v) { return v > -6 || isNaN(v); }));");
trace(b_number.filter(function (v) { return v > -6 || isNaN(v); }));

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.filter(function (v) { return v.length > 0; }));");
trace(a_string.filter(function (v) { return v.length > 0; }));

trace("/// a_string.filter(function (v) { return v.length > 2; }));");
trace(a_string.filter(function (v) { return v.length > 2; }));

trace("/// b_string.filter(function (v) { return v.length > 0; }));");
trace(b_string.filter(function (v) { return v.length > 0; }));

trace("/// b_string.filter(function (v) { return v.length > 4; }));");
trace(b_string.filter(function (v) { return v.length > 4; }));

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.filter(function (v) { return v > 0; }));");
trace(a_uint.filter(function (v) { return v > 0; }));

trace("/// a_uint.filter(function (v) { return v > 2; }));");
trace(a_uint.filter(function (v) { return v > 2; }));

trace("/// b_uint.filter(function (v) { return v > 4; }));");
trace(b_uint.filter(function (v) { return v > 4; }));

trace("/// b_uint.filter(function (v) { return v > 10; }));");
trace(b_uint.filter(function (v) { return v > 10; }));

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// a_vector.filter(function (v) { return v.filter(function (v) { return v > 0; }).length > 0; });");
trace(a_vector.filter(function (v) { return v.filter(function (v) { return v > 0; }).length > 0; }));

trace("/// a_vector.filter(function (v) { return v.filter(function (v) { return v > 2; }).length > 0; });");
trace(a_vector.filter(function (v) { return v.filter(function (v) { return v > 2; }).length > 0; }));

trace("/// b_vector.filter(function (v) { return v.filter(function (v) { return v > 4; }).length > 0; });");
trace(b_vector.filter(function (v) { return v.filter(function (v) { return v > 4; }).length > 0; }));

trace("/// b_vector.filter(function (v) { return v.filter(function (v) { return v > 25; }).length > 0; });");
trace(b_vector.filter(function (v) { return v.filter(function (v) { return v > 25; }).length > 0; }));