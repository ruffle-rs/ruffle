package {
	public class Test {
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[true, false];");
var a_bool:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b_bool:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// a_bool.join('...');");
trace(a_bool.join("..."));

trace("/// b_bool.join('...');");
trace(b_bool.join("..."));

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

trace("/// a_class.join('...');");
trace(a_class.join("..."));

trace("/// b_class.join('...');");
trace(b_class.join("..."));

interface Interface {
	
}

class Implementation implements Interface {
	
}

trace("/// var a_iface: Vector.<Interface> = new <Interface>[];");
var a_iface:Vector.<Interface> = new <Interface>[];

trace("/// a_iface.length = 1;");
a_iface.length = 1;

trace("/// a_iface[0] = new Implementation();");
a_iface[0] = new Implementation();

trace("/// var b_iface: Vector.<Implementation> = new <Implementation>[];");
var b_iface:Vector.<Implementation> = new <Implementation>[];

trace("/// b_iface.length = 2;");
b_iface.length = 2;

trace("/// b_iface[0] = new Implementation();");
b_iface[0] = new Implementation();

trace("/// b_iface[1] = new Implementation();");
b_iface[1] = new Implementation();

trace("/// a_iface.join('...');");
trace(a_iface.join("..."));

trace("/// b_iface.join('...');");
trace(b_iface.join("..."));

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.join('...');");
trace(a_int.join("..."));

trace("/// b_int.join('...');");
trace(b_int.join("..."));

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.join('...');");
trace(a_number.join("..."));

trace("/// b_number.join('...');");
trace(b_number.join("..."));

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.join('...');");
trace(a_string.join("..."));

trace("/// b_string.join('...');");
trace(b_string.join("..."));

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.join('...');");
trace(a_uint.join("..."));

trace("/// b_uint.join('...');");
trace(b_uint.join("..."));

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// a_vector.join('...');");
trace(a_vector.join("..."));

trace("/// b_vector.join('...');");
trace(b_vector.join("..."));