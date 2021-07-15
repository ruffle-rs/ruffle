package {
	public class Test {
	}
}

function trace_vector(v) {
	trace("///length: ", v.length);
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[true, false];");
var a_bool:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b_bool:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// var c_bool = a_bool.concat(b_bool);");
var c_bool = a_bool.concat(b_bool);

trace("/// (contents of c_bool...)");
trace_vector(c_bool);

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

trace("/// var c_class = a_class.concat(b_class);");
var c_class = a_class.concat(b_class);

trace("/// (contents of c_class...)");
trace_vector(c_class);

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

trace("/// b_iface.length = 1;");
b_iface.length = 1;

trace("/// b_iface[0] = new Implementer();");
b_iface[0] = new Implementer();

trace("/// var c_iface = a_iface.concat(b_iface);");
var c_iface = a_iface.concat(b_iface);

trace("/// (contents of c_iface...)");
trace_vector(c_iface);

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// var c_int = a_int.concat(b_int);");
var c_int = a_int.concat(b_int);

trace("/// (contents of c_int...)");
trace_vector(c_int);

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// var c_number = a_number.concat(b_number);");
var c_number = a_number.concat(b_number);

trace("/// (contents of c_number...)");
trace_vector(c_number);

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// var c_string = a_string.concat(b_string);");
var c_string = a_string.concat(b_string);

trace("/// (contents of c_string...)");
trace_vector(c_string);

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// var c_uint = a_uint.concat(b_uint);");
var c_uint = a_uint.concat(b_uint);

trace("/// (contents of c_uint...)");
trace_vector(c_uint);

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16]];

trace("/// var c_vector = a_vector.concat(b_vector)");
var c_vector = a_vector.concat(b_vector);

trace("/// (contents of c_vector...)");
trace_vector(c_vector);