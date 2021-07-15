package {
	public class Test {
	}
}

trace("/// var a_bool: Vector.<Boolean> = new <Boolean>[true, false];");
var a_bool:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b_bool: Vector.<Boolean> = new <Boolean>[true, true];");
var b_bool:Vector.<Boolean> = new <Boolean>[true, true];

trace("/// a_bool.lastIndexOf(true);");
trace(a_bool.lastIndexOf(true));

trace("/// a_bool.lastIndexOf(false));");
trace(a_bool.lastIndexOf(false));

trace("/// b_bool.lastIndexOf(true);");
trace(b_bool.lastIndexOf(true));

trace("/// b_bool.lastIndexOf(false);");
trace(b_bool.lastIndexOf(false));

trace("/// a_bool.lastIndexOf(true, 1);");
trace(a_bool.lastIndexOf(true, 1));

trace("/// a_bool.lastIndexOf(false, 1));");
trace(a_bool.lastIndexOf(false, 1));

trace("/// b_bool.lastIndexOf(true, 1);");
trace(b_bool.lastIndexOf(true, 1));

trace("/// b_bool.lastIndexOf(false, 1);");
trace(b_bool.lastIndexOf(false, 1));

trace("/// a_bool.lastIndexOf(true, 0);");
trace(a_bool.lastIndexOf(true, 0));

trace("/// a_bool.lastIndexOf(false, 0));");
trace(a_bool.lastIndexOf(false, 0));

trace("/// b_bool.lastIndexOf(true, 0);");
trace(b_bool.lastIndexOf(true, 0));

trace("/// b_bool.lastIndexOf(false, 0);");
trace(b_bool.lastIndexOf(false, 0));

class Superclass {
	
}

class Subclass extends Superclass {
	
}

trace("/// var a_class: Vector.<Superclass> = new <Superclass>[];");
var a_class:Vector.<Superclass> = new <Superclass>[];

trace("/// a_class.length = 2;");
a_class.length = 2;

trace("/// var a0_class = new Superclass();");
var a0_class = new Superclass();

trace("/// a_class[0] = a0_class;");
a_class[0] = a0_class;

trace("/// var a1_class = new Subclass();");
var a1_class = new Subclass();

trace("/// a_class[1] = a1_class;");
a_class[1] = a1_class;

trace("/// var b_class: Vector.<Subclass> = new <Subclass>[];");
var b_class:Vector.<Subclass> = new <Subclass>[];

trace("/// b_class.length = 1;");
b_class.length = 1;

trace("/// var b0_class = new Subclass();");
var b0_class = new Subclass();

trace("/// b_class[0] = b0_class;");
b_class[0] = b0_class;

trace("/// a_class.lastIndexOf(a0_class);");
trace(a_class.lastIndexOf(a0_class));

trace("/// a_class.lastIndexOf(a1_class);");
trace(a_class.lastIndexOf(a1_class));

trace("/// a_class.lastIndexOf(b0_class);");
trace(a_class.lastIndexOf(b0_class));

trace("/// b_class.lastIndexOf(a0_class);");
trace(b_class.lastIndexOf(a0_class));

trace("/// b_class.lastIndexOf(a1_class);");
trace(b_class.lastIndexOf(a1_class));

trace("/// b_class.lastIndexOf(b0_class);");
trace(b_class.lastIndexOf(b0_class));

trace("/// a_class.lastIndexOf(a0_class, 0);");
trace(a_class.lastIndexOf(a0_class, 0));

trace("/// a_class.lastIndexOf(a1_class, 0);");
trace(a_class.lastIndexOf(a1_class, 0));

trace("/// a_class.lastIndexOf(b0_class, 0);");
trace(a_class.lastIndexOf(b0_class, 0));

trace("/// b_class.lastIndexOf(a0_class, 0);");
trace(b_class.lastIndexOf(a0_class, 0));

trace("/// b_class.lastIndexOf(a1_class, 0);");
trace(b_class.lastIndexOf(a1_class, 0));

trace("/// b_class.lastIndexOf(b0_class, 0);");
trace(b_class.lastIndexOf(b0_class, 0));

trace("/// a_class.lastIndexOf(a0_class, -1);");
trace(a_class.lastIndexOf(a0_class, -1));

trace("/// a_class.lastIndexOf(a1_class, -1);");
trace(a_class.lastIndexOf(a1_class, -1));

trace("/// a_class.lastIndexOf(b0_class, -1);");
trace(a_class.lastIndexOf(b0_class, -1));

trace("/// b_class.lastIndexOf(a0_class, -1);");
trace(b_class.lastIndexOf(a0_class, -1));

trace("/// b_class.lastIndexOf(a1_class, -1);");
trace(b_class.lastIndexOf(a1_class, -1));

trace("/// b_class.lastIndexOf(b0_class, -1);");
trace(b_class.lastIndexOf(b0_class, -1));

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16];");
var b_int:Vector.<int> = new <int>[5,16];

trace("/// a_int.lastIndexOf(0);");
trace(a_int.lastIndexOf(0));

trace("/// a_int.lastIndexOf(1);");
trace(a_int.lastIndexOf(1));

trace("/// a_int.lastIndexOf(2);");
trace(a_int.lastIndexOf(2));

trace("/// b_int.lastIndexOf(3);");
trace(b_int.lastIndexOf(3));

trace("/// b_int.lastIndexOf(5);");
trace(b_int.lastIndexOf(5));

trace("/// b_int.lastIndexOf(15);");
trace(b_int.lastIndexOf(15));

trace("/// a_int.lastIndexOf(0, 0);");
trace(a_int.lastIndexOf(0, 0));

trace("/// a_int.lastIndexOf(1, 0);");
trace(a_int.lastIndexOf(1, 0));

trace("/// a_int.lastIndexOf(2, 0);");
trace(a_int.lastIndexOf(2, 0));

trace("/// b_int.lastIndexOf(3, 0);");
trace(b_int.lastIndexOf(3, 0));

trace("/// b_int.lastIndexOf(5, 0);");
trace(b_int.lastIndexOf(5, 0));

trace("/// b_int.lastIndexOf(15, 0);");
trace(b_int.lastIndexOf(15, 0));

trace("/// a_int.lastIndexOf(0, -2);");
trace(a_int.lastIndexOf(0, -2));

trace("/// a_int.lastIndexOf(1, -2);");
trace(a_int.lastIndexOf(1, -2));

trace("/// a_int.lastIndexOf(2, -2);");
trace(a_int.lastIndexOf(2, -2));

trace("/// b_int.lastIndexOf(3, -2);");
trace(b_int.lastIndexOf(3, -2));

trace("/// b_int.lastIndexOf(5, -2);");
trace(b_int.lastIndexOf(5, -2));

trace("/// b_int.lastIndexOf(15, -2);");
trace(b_int.lastIndexOf(15, -2));

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0];

trace("/// a_number.lastIndexOf(0);");
trace(a_number.lastIndexOf(0));

trace("/// a_number.lastIndexOf(1);");
trace(a_number.lastIndexOf(1));

trace("/// a_number.lastIndexOf(2);");
trace(a_number.lastIndexOf(2));

trace("/// b_number.lastIndexOf(3);");
trace(b_number.lastIndexOf(3));

trace("/// b_number.lastIndexOf(-5);");
trace(b_number.lastIndexOf(-5));

trace("/// b_number.lastIndexOf(NaN);");
trace(b_number.lastIndexOf(NaN));

trace("/// a_number.lastIndexOf(0, 1);");
trace(a_number.lastIndexOf(0, 1));

trace("/// a_number.lastIndexOf(1, 1);");
trace(a_number.lastIndexOf(1, 1));

trace("/// a_number.lastIndexOf(2, 1);");
trace(a_number.lastIndexOf(2, 1));

trace("/// b_number.lastIndexOf(3, 1);");
trace(b_number.lastIndexOf(3, 1));

trace("/// b_number.lastIndexOf(-5, 1);");
trace(b_number.lastIndexOf(-5, 1));

trace("/// b_number.lastIndexOf(NaN, 1);");
trace(b_number.lastIndexOf(NaN, 1));

trace("/// a_number.lastIndexOf(0, -2);");
trace(a_number.lastIndexOf(0, -2));

trace("/// a_number.lastIndexOf(1, -2);");
trace(a_number.lastIndexOf(1, -2));

trace("/// a_number.lastIndexOf(2, -2);");
trace(a_number.lastIndexOf(2, -2));

trace("/// b_number.lastIndexOf(3, -2);");
trace(b_number.lastIndexOf(3, -2));

trace("/// b_number.lastIndexOf(-5, -2);");
trace(b_number.lastIndexOf(-5, -2));

trace("/// b_number.lastIndexOf(NaN, -2);");
trace(b_number.lastIndexOf(NaN, -2));

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr"];

trace("/// a_string.lastIndexOf(\"a\");");
trace(a_string.lastIndexOf("a"));

trace("/// a_string.lastIndexOf(\"z\");");
trace(a_string.lastIndexOf("z"));

trace("/// a_string.lastIndexOf(\"d\");");
trace(a_string.lastIndexOf("d"));

trace("/// b_string.lastIndexOf(986);");
trace(b_string.lastIndexOf(986));

trace("/// b_string.lastIndexOf(\"986\");");
trace(b_string.lastIndexOf("986"));

trace("/// b_string.lastIndexOf(\"Q\");");
trace(b_string.lastIndexOf("Q"));

trace("/// a_string.lastIndexOf(\"a\", -2);");
trace(a_string.lastIndexOf("a", -2));

trace("/// a_string.lastIndexOf(\"z\", -2);");
trace(a_string.lastIndexOf("z", -2));

trace("/// a_string.lastIndexOf(\"d\", -2);");
trace(a_string.lastIndexOf("d", -2));

trace("/// b_string.lastIndexOf(986, -2);");
trace(b_string.lastIndexOf(986, -2));

trace("/// b_string.lastIndexOf(\"986\", -2);");
trace(b_string.lastIndexOf("986", -2));

trace("/// b_string.lastIndexOf(\"Q\", -2);");
trace(b_string.lastIndexOf("Q", -2));

trace("/// a_string.lastIndexOf(\"a\", 2);");
trace(a_string.lastIndexOf("a", 2));

trace("/// a_string.lastIndexOf(\"z\", 2);");
trace(a_string.lastIndexOf("z", 2));

trace("/// a_string.lastIndexOf(\"d\", 2);");
trace(a_string.lastIndexOf("d", 2));

trace("/// b_string.lastIndexOf(986, 2);");
trace(b_string.lastIndexOf(986, 2));

trace("/// b_string.lastIndexOf(\"986\", 2);");
trace(b_string.lastIndexOf("986", 2));

trace("/// b_string.lastIndexOf(\"Q\", 2);");
trace(b_string.lastIndexOf("Q", 2));

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16];");
var b_uint:Vector.<uint> = new <uint>[5,16];

trace("/// a_uint.lastIndexOf(0);");
trace(a_uint.lastIndexOf(0));

trace("/// a_uint.lastIndexOf(1);");
trace(a_uint.lastIndexOf(1));

trace("/// a_uint.lastIndexOf(2);");
trace(a_uint.lastIndexOf(2));

trace("/// b_uint.lastIndexOf(3);");
trace(b_uint.lastIndexOf(3));

trace("/// b_uint.lastIndexOf(5);");
trace(b_uint.lastIndexOf(5));

trace("/// b_uint.lastIndexOf(12);");
trace(b_uint.lastIndexOf(12));

trace("/// a_uint.lastIndexOf(0, 1);");
trace(a_uint.lastIndexOf(0, 1));

trace("/// a_uint.lastIndexOf(1, 1);");
trace(a_uint.lastIndexOf(1, 1));

trace("/// a_uint.lastIndexOf(2, 1);");
trace(a_uint.lastIndexOf(2, 1));

trace("/// b_uint.lastIndexOf(3, 1);");
trace(b_uint.lastIndexOf(3, 1));

trace("/// b_uint.lastIndexOf(5, 1);");
trace(b_uint.lastIndexOf(5, 1));

trace("/// b_uint.lastIndexOf(12, 1);");
trace(b_uint.lastIndexOf(12, 1));

trace("/// a_uint.lastIndexOf(0, -1);");
trace(a_uint.lastIndexOf(0, -1));

trace("/// a_uint.lastIndexOf(1, -1);");
trace(a_uint.lastIndexOf(1, -1));

trace("/// a_uint.lastIndexOf(2, -1);");
trace(a_uint.lastIndexOf(2, -1));

trace("/// b_uint.lastIndexOf(3, -1);");
trace(b_uint.lastIndexOf(3, -1));

trace("/// b_uint.lastIndexOf(5, -1);");
trace(b_uint.lastIndexOf(5, -1));

trace("/// b_uint.lastIndexOf(12, -1);");
trace(b_uint.lastIndexOf(12, -1));

trace("/// var a0_vector = new <int>[1,2];");
var a0_vector = new <int>[1,2];

trace("/// var a1_vector = new <int>[4,3];");
var a1_vector = new <int>[4,3];

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[a0_vector, a1_vector];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[a0_vector, a1_vector];

trace("/// var b0_vector = new <int>[5,16];");
var b0_vector = new <int>[5,16];

trace("/// var b1_vector = new <int>[19,8];");
var b1_vector = new <int>[19,8];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[b0_vector, b1_vector];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[b0_vector, b1_vector];

trace("/// a_vector.lastIndexOf(a0_vector)");
trace(a_vector.lastIndexOf(a0_vector));

trace("/// a_vector.lastIndexOf(a1_vector)");
trace(a_vector.lastIndexOf(a1_vector));

trace("/// a_vector.lastIndexOf(new <int>[4,3])");
trace(a_vector.lastIndexOf(new <int>[4, 3]));

trace("/// a_vector.lastIndexOf(b0_vector)");
trace(a_vector.lastIndexOf(b0_vector));

trace("/// a_vector.lastIndexOf(b1_vector)");
trace(a_vector.lastIndexOf(b1_vector));

trace("/// a_vector.lastIndexOf(new <int>[19,8])");
trace(a_vector.lastIndexOf(new <int>[19,8]));

trace("/// b_vector.lastIndexOf(a0_vector)");
trace(b_vector.lastIndexOf(a0_vector));

trace("/// b_vector.lastIndexOf(a1_vector)");
trace(b_vector.lastIndexOf(a1_vector));

trace("/// b_vector.lastIndexOf(new <int>[4,3])");
trace(b_vector.lastIndexOf(new <int>[4, 3]));

trace("/// b_vector.lastIndexOf(b0_vector)");
trace(b_vector.lastIndexOf(b0_vector));

trace("/// b_vector.lastIndexOf(b1_vector)");
trace(b_vector.lastIndexOf(b1_vector));

trace("/// b_vector.lastIndexOf(new <int>[19,8])");
trace(b_vector.lastIndexOf(new <int>[19,8]));

trace("/// a_vector.lastIndexOf(a0_vector, 0)");
trace(a_vector.lastIndexOf(a0_vector, 0));

trace("/// a_vector.lastIndexOf(a1_vector, 0)");
trace(a_vector.lastIndexOf(a1_vector, 0));

trace("/// a_vector.lastIndexOf(new <int>[4,3], 0)");
trace(a_vector.lastIndexOf(new <int>[4, 3], 0));

trace("/// a_vector.lastIndexOf(b0_vector, 0)");
trace(a_vector.lastIndexOf(b0_vector, 0));

trace("/// a_vector.lastIndexOf(b1_vector, 0)");
trace(a_vector.lastIndexOf(b1_vector, 0));

trace("/// a_vector.lastIndexOf(new <int>[19,8], 0)");
trace(a_vector.lastIndexOf(new <int>[19,8], 0));

trace("/// b_vector.lastIndexOf(a0_vector, 0)");
trace(b_vector.lastIndexOf(a0_vector, 0));

trace("/// b_vector.lastIndexOf(a1_vector, 0)");
trace(b_vector.lastIndexOf(a1_vector, 0));

trace("/// b_vector.lastIndexOf(new <int>[4,3], 0)");
trace(b_vector.lastIndexOf(new <int>[4, 3], 0));

trace("/// b_vector.lastIndexOf(b0_vector, 0)");
trace(b_vector.lastIndexOf(b0_vector, 0));

trace("/// b_vector.lastIndexOf(b1_vector, 0)");
trace(b_vector.lastIndexOf(b1_vector, 0));

trace("/// b_vector.lastIndexOf(new <int>[19,8], 0)");
trace(b_vector.lastIndexOf(new <int>[19,8], 0));

trace("/// a_vector.lastIndexOf(a0_vector, -1)");
trace(a_vector.lastIndexOf(a0_vector, -1));

trace("/// a_vector.lastIndexOf(a1_vector, -1)");
trace(a_vector.lastIndexOf(a1_vector, -1));

trace("/// a_vector.lastIndexOf(new <int>[4,3], -1)");
trace(a_vector.lastIndexOf(new <int>[4, 3], -1));

trace("/// a_vector.lastIndexOf(b0_vector, -1)");
trace(a_vector.lastIndexOf(b0_vector, -1));

trace("/// a_vector.lastIndexOf(b1_vector, -1)");
trace(a_vector.lastIndexOf(b1_vector, -1));

trace("/// a_vector.lastIndexOf(new <int>[19,8], -1)");
trace(a_vector.lastIndexOf(new <int>[19,8], -1));

trace("/// b_vector.lastIndexOf(a0_vector, -1)");
trace(b_vector.lastIndexOf(a0_vector, -1));

trace("/// b_vector.lastIndexOf(a1_vector, -1)");
trace(b_vector.lastIndexOf(a1_vector, -1));

trace("/// b_vector.lastIndexOf(new <int>[4,3], -1)");
trace(b_vector.lastIndexOf(new <int>[4, 3], -1));

trace("/// b_vector.lastIndexOf(b0_vector, -1)");
trace(b_vector.lastIndexOf(b0_vector, -1));

trace("/// b_vector.lastIndexOf(b1_vector, -1)");
trace(b_vector.lastIndexOf(b1_vector, -1));

trace("/// b_vector.lastIndexOf(new <int>[19,8], -1)");
trace(b_vector.lastIndexOf(new <int>[19,8], -1));