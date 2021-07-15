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

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of a_bool.sort(0)...)");
trace_vector(a_bool.sort(0));

trace("/// (contents of a_bool.sort(Array.DESCENDING)...)");
trace_vector(a_bool.sort(Array.DESCENDING));

trace("/// (contents of a_bool...)");
trace_vector(a_bool);

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of b_bool.sort(0)...)");
trace_vector(b_bool.sort(0));

trace("/// (contents of b_bool.sort(Array.RETURNINDEXEDARRAY)...)");
trace_vector(b_bool.sort(Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_bool...)");
trace_vector(b_bool);

trace("/// (contents of c_bool.sort(0)...)");
trace_vector(c_bool.sort(0));

trace("/// (contents of c_bool...)");
trace_vector(c_bool);

class Superclass {
	private var v = "[object Superclass]";
	
	public function Superclass(v: String) {
		if (v) {
			this.v = v;
		}
	}
	
	public function toString() {
		return this.v;
	}
	
	public function valueOf() {
		return this.v;
	}
}

class Subclass extends Superclass {
	public function Subclass(v: String) {
		if (!v) {
			v = "[object Subclass]";
		}
		
		super(v);
	}
}

trace("/// var a0_class = new Superclass(\"\");");
var a0_class = new Superclass("");

trace("/// var a1_class = new Subclass(\"\");");
var a1_class = new Subclass("");

trace("/// var a_class: Vector.<Superclass> = new <Superclass>[a0_class, a1_class];");
var a_class:Vector.<Superclass> = new <Superclass>[a0_class, a1_class];

trace("/// var b_class: Vector.<Subclass> = new <Subclass>[];");
var b_class:Vector.<Subclass> = new <Subclass>[];

trace("/// b_class.length = 1;");
b_class.length = 1;

trace("/// var b0_class = new Subclass(\"\");");
var b0_class = new Subclass("");

trace("/// b_class[0] = b0_class;");
b_class[0] = b0_class;

trace("/// var c0_class = new Superclass(\"10\");");
var c0_class = new Superclass("10");

trace("/// var c1_class = new Subclass(\"11\");");
var c1_class = new Subclass("11");

trace("/// var c2_class = new Subclass(\"12\");");
var c2_class = new Subclass("12");

trace("/// var c3_class = new Subclass(\"13\");");
var c3_class = new Subclass("13");

trace("/// var c4_class = new Subclass(\"14\");");
var c4_class = new Subclass("14");

trace("/// var c_class: Vector.<Superclass> = new <Superclass>[c4_class, c1_class, c2_class, c3_class, c0_class];");
var c_class:Vector.<Superclass> = new <Superclass>[c4_class, c1_class, c2_class, c3_class, c0_class];

trace("/// (contents of a_class.sort(0)...)");
trace_vector(a_class.sort(0));

trace("/// a0_class === a_class[0];");
trace(a0_class === a_class[0]);

trace("/// a1_class === a_class[0];");
trace(a1_class === a_class[0]);

trace("/// a0_class === a_class[1];");
trace(a0_class === a_class[1]);

trace("/// a1_class === a_class[1];");
trace(a1_class === a_class[1]);

trace("/// (contents of a_class...)");
trace_vector(a_class);

trace("/// (contents of b_class.sort(0)");
trace_vector(b_class.sort(0));

trace("/// b_class === b_class[0];");
trace(b_class === b_class[0]);

trace("/// (contents of b_class...)");
trace_vector(b_class);

trace("/// b_class === b_class.sort(0);");
trace(b_class === b_class.sort(0));

trace("/// var c_sorted = c_class.sort(Array.RETURNINDEXEDARRAY | Array.NUMERIC);");
var c_sorted = c_class.sort(Array.RETURNINDEXEDARRAY | Array.NUMERIC);

trace("/// c_class === c_sorted");
trace(c_class === c_sorted);

trace("/// (contents of c_sorted...)");
trace_vector(c_sorted);

trace("/// c_sorted is Vector.<int>;");
trace(c_sorted is Vector.<int>);

trace("/// c_sorted is Vector.<Superclass>;");
trace(c_sorted is Vector.<Superclass>);

trace("/// c_sorted = c_class.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY | Array.NUMERIC);");
c_sorted = c_class.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY | Array.NUMERIC);

trace("/// (contents of c_sorted...)");
trace_vector(c_sorted);

trace("/// c_sorted = c_class.sort(Array.RETURNINDEXEDARRAY);");
c_sorted = c_class.sort(Array.RETURNINDEXEDARRAY);

trace("/// (contents of c_sorted...)");
trace_vector(c_sorted);

trace("/// c_sorted = c_class.sort(0);");
c_sorted = c_class.sort(0);

trace("/// (contents of c_sorted...)");
trace_vector(c_sorted);

function trace_vector_int(v: Vector.<int>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_int: Vector.<int> = new <int>[1,2];");
var a_int:Vector.<int> = new <int>[1,2];

trace("/// var b_int: Vector.<int> = new <int>[5,16,8,24,8];");
var b_int:Vector.<int> = new <int>[5,16,8,24,8];

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_int.sort(Array.NUMERIC))...");
trace_vector_int(a_int.sort(Array.NUMERIC));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(a_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_int.sort(Array.RETURNINDEXEDARRAY))...");
trace_vector_int(b_int.sort(Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_int.sort(Array.NUMERIC))...");
trace_vector_int(b_int.sort(Array.NUMERIC));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_int(b_int.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

function trace_vector_number(v: Vector.<Number>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_number: Vector.<Number> = new <Number>[1,2,3,4,5];");
var a_number:Vector.<Number> = new <Number>[1,2,3,4,5];

trace("/// var b_number: Vector.<Number> = new <Number>[5, NaN, -5, 0, NaN];");
var b_number:Vector.<Number> = new <Number>[5, NaN, -5, 0, NaN];

trace("/// (contents of a_number.sort(Array.NUMERIC))...");
trace_vector_number(a_number.sort(Array.NUMERIC));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(a_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_number.sort(Array.NUMERIC))...");
trace_vector_number(b_number.sort(Array.NUMERIC));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_number(b_number.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

function trace_vector_string(v: Vector.<String>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_string: Vector.<String> = new <String>[\"a\",\"c\",\"d\",\"f\",\"-1\"];");
var a_string:Vector.<String> = new <String>["a", "c", "d", "f", "-1"];

trace("/// var b_string: Vector.<String> = new <String>[\"986\",\"B4\",\"Q\",\"rrr\",\"q\"];");
var b_string:Vector.<String> = new <String>["986", "B4", "Q", "rrr", "q"];

trace("/// var c_string: Vector.<String> = new <String>[\"986\",\"4\",\"13\",\"12.5\",\"1\"];");
var c_string:Vector.<String> = new <String>["986", "4", "13", "12.5", "1"];

trace("/// (contents of a_string.sort(0))...");
trace_vector_string(a_string.sort(0));

trace("/// (contents of a_string.sort(Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.DESCENDING))...");
trace_vector_string(a_string.sort(Array.DESCENDING));

trace("/// (contents of a_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.UNIQUESORT))...");
trace_vector_string(a_string.sort(Array.UNIQUESORT));

trace("/// (contents of a_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(a_string.sort(Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of a_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(a_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(0))...");
trace_vector_string(b_string.sort(0));

trace("/// (contents of b_string.sort(Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.DESCENDING))...");
trace_vector_string(b_string.sort(Array.DESCENDING));

trace("/// (contents of b_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.UNIQUESORT))...");
trace_vector_string(b_string.sort(Array.UNIQUESORT));

trace("/// (contents of b_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(b_string.sort(Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of b_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(b_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(0))...");
trace_vector_string(c_string.sort(0));

trace("/// (contents of c_string.sort(Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.UNIQUESORT))...");
trace_vector_string(c_string.sort(Array.UNIQUESORT));

trace("/// (contents of c_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.NUMERIC))...");
trace_vector_string(c_string.sort(Array.NUMERIC));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_string(c_string.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

function trace_vector_uint(v: Vector.<uint>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a_uint: Vector.<uint> = new <uint>[1,2];");
var a_uint:Vector.<uint> = new <uint>[1,2];

trace("/// var b_uint: Vector.<uint> = new <uint>[5,16,32,8,128,5];");
var b_uint:Vector.<uint> = new <uint>[5,16,32,8,128,5];

trace("/// (contents of a_uint.sort(Array.NUMERIC))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(a_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_uint.sort(Array.NUMERIC))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.DESCENDING))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.DESCENDING));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.UNIQUESORT))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.UNIQUESORT));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING));

trace("/// (contents of b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY))...");
trace_vector_uint(b_uint.sort(Array.NUMERIC | Array.UNIQUESORT | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

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

function cmp_vector_len(x: Vector.<int>, y: Vector.<int>):Number {
	return y.length - x.length;
}

function cmp_vector_sum(x: Vector.<int>, y: Vector.<int>):Number {
	var sum_x = 0;
	var sum_y = 0;
	
	for (var i = 0; i < x.length && i < y.length; i += 1) {
		sum_x += x[i];
		sum_y += y[i];
	}
	
	return sum_y - sum_x;
}

trace("/// var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[8,2], new <int>[4,3,0]];");
var a_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[8,2], new <int>[4,3,0]];

trace("/// var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16,1], new <int>[19,863]];");
var b_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16,1], new <int>[19,863]];

trace("/// var c_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16,1], new <int>[19,8], new <int>[1,8], new <int>[]];");
var c_vector:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16,1], new <int>[19,8], new <int>[1,8], new <int>[]];

trace("/// (contents of a_vector.sort() custom-sorted by length...)");
trace_vector_vector(a_vector.sort(cmp_vector_len));

trace("/// (contents of a_vector.sort() custom-sorted by sum...)");
trace_vector_vector(a_vector.sort(cmp_vector_sum));

trace("/// (contents of b_vector.sort() custom-sorted by length...)");
trace_vector_vector(b_vector.sort(cmp_vector_len));

trace("/// (contents of b_vector.sort() custom-sorted by sum...)");
trace_vector_vector(b_vector.sort(cmp_vector_sum));

trace("/// (contents of c_vector.sort() custom-sorted by length...)");
trace_vector_vector(c_vector.sort(cmp_vector_len));

trace("/// (contents of c_vector.sort() custom-sorted by sum...)");
trace_vector_vector(c_vector.sort(cmp_vector_sum));