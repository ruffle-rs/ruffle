package {
	public class Test {
	}
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

function length_based_comparison(a, b) {
	if ("length" in a) {
		if ("length" in b) {
			return a.length - b.length;
		} else {
			return a.length - b;
		}
	} else {
		if ("length" in b) {
			return a - b.length;
		} else {
			return a - b;
		}
	}
}

function sub_comparison(a, b) {
	return a - b;
}

function lbc(a, b) {
	trace(a);
	trace(b);
	var x = length_based_comparison(a, b);
	trace(x);
	return x;
}

function sc(a, b) {
	trace(a);
	trace(b);
	var x = sub_comparison(a, b);
	trace(x);
	return x;
}

function fresh_array() {
	trace("//var a = new Array(5,3,1,\"Abc\",\"2\",\"aba\",false,null,\"zzz\")");
	var a = new Array(5,3,1,"Abc","2","aba",false,null,"zzz");

	trace("//a[11] = \"not a hole\";");
	a[11] = "not a hole";
	
	return a;
}

function fresh_array_b() {
	trace("//var b = new Array(5,3,\"2\",false,true,NaN)");
	var b = new Array(5,3,"2",false,true,NaN);
	
	return b;
}

function check_holes(a) {
	trace("//Array.prototype[10] = \"hole10\";");
	Array.prototype[10] = "hole10";

	trace("//Array.prototype[11] = \"hole11\";");
	Array.prototype[11] = "hole11";

	trace("//Array.prototype[12] = \"hole12\";");
	Array.prototype[12] = "hole12";

	trace("//(contents of previous array)");
	assert_array(a);

	trace("//(cleaning up our holes...)");
	delete Array.prototype[10];
	delete Array.prototype[11];
	delete Array.prototype[12];
	
	trace("//Array.prototype[9] = undefined;");
	Array.prototype[9] = undefined;
	
	trace("//Array.prototype[10] = \"hole in slot 10\";");
	Array.prototype[10] = "hole in slot 10";
}

var a = fresh_array();

trace("//Array.prototype[9] = undefined;");
Array.prototype[9] = undefined;

trace("//Array.prototype[10] = \"hole in slot 10\";");
Array.prototype[10] = "hole in slot 10";

trace("//a.sort(Array.UNIQUESORT) === 0");
trace(a.sort(Array.UNIQUESORT) === 0);

a = fresh_array();

trace("//(contents of a.sort(Array.RETURNINDEXEDARRAY))");
assert_array(a.sort(Array.RETURNINDEXEDARRAY));

trace("//(contents of a.sort())");
assert_array(a.sort());
check_holes(a);

a = fresh_array();

trace("//(contents of a.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY))");
assert_array(a.sort(Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY));

trace("//(contents of a.sort(Array.CASEINSENSITIVE))");
assert_array(a.sort(Array.CASEINSENSITIVE));
check_holes(a);

a = fresh_array();

trace("//(contents of a.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY))");
assert_array(a.sort(Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("//(contents of a.sort(Array.DESCENDING))");
assert_array(a.sort(Array.DESCENDING));
check_holes(a);

a = fresh_array();

trace("//(contents of a.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY))");
assert_array(a.sort(Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("//(contents of a.sort(Array.CASEINSENSITIVE | Array.DESCENDING))");
assert_array(a.sort(Array.CASEINSENSITIVE | Array.DESCENDING));
check_holes(a);

a = fresh_array();

trace("//var b = new Array(5,3,2,1,\"2\",false,true,NaN)");
var b = new Array(5,3,2,1,"2",false,true,NaN);

trace("//b.sort(Array.NUMERIC | Array.UNIQUESORT) === 0");
trace(b.sort(Array.NUMERIC | Array.UNIQUESORT) === 0);

b = fresh_array_b();

trace("//(contents of b.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY))");
assert_array(b.sort(Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("//(contents of b.sort(Array.NUMERIC))");
assert_array(b.sort(Array.NUMERIC));
check_holes(b);

b = fresh_array_b();

trace("//(contents of b.sort(Array.NUMERIC | 1))");
assert_array(b.sort(Array.NUMERIC | 1));

b = fresh_array_b();

trace("//(contents of b.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY))");
assert_array(b.sort(Array.NUMERIC | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("//(contents of b.sort(16 | Array.DESCENDING))");
assert_array(b.sort(16 | Array.DESCENDING));
check_holes(b);

trace("//var a = new Array(7,2,1,\"3\",\"4\")");
var a = new Array(7,2,1,"3","4");

trace("//(contents of a.sort(sub_comparison))");
assert_array(a.sort(sub_comparison));

trace("//(contents of a.sort(sub_comparison, 2))");
assert_array(a.sort(sub_comparison, 2));

trace("//(contents of a.sort(sub_comparison, Array.RETURNINDEXEDARRAY))");
assert_array(a.sort(sub_comparison, Array.RETURNINDEXEDARRAY));

trace("//(contents of a.sort(sub_comparison, Array.DESCENDING | 8))");
assert_array(a.sort(sub_comparison, Array.DESCENDING | 8));

trace("//a.sort(sub_comparison, Array.UNIQUESORT) === 0");
trace(a.sort(sub_comparison, Array.UNIQUESORT) === 0);

trace("//var c = new Array(3,\"abc\")");
var c = new Array(3,"abc");

trace("//c.sort(sub_comparison, Array.UNIQUESORT) === 0");
trace(c.sort(sub_comparison, Array.UNIQUESORT) === 0);

trace("//var d = new Array(3,\"4\")");
var d = new Array(3,"4");

trace("//(contents of d.sort(sub_comparison, 4))");
assert_array(d.sort(sub_comparison, 4));