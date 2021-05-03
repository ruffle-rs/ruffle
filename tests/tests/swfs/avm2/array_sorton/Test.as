package {
	public class Test {
	}
}

function assert_array(a) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

function assert_array_props(a) {
	for (var i = 0; i < a.length; i += 1) {
		if (a[i] !== undefined && a[i] !== null) {
			trace(a[i].numprop);
			trace(a[i].strprop);
		} else {
			trace("//(undefined value omitted)");
		}
	}
}

function fresh_array_a() {
	trace("//var a = new Array(item1, item2, item3)");
	var a = new Array(item1, item2, item3);
	
	trace("//a[4] = item5;");
	a[4] = item5;
	
	return a;
}

function test_holes(a) {
	var item4 = {"numprop": 9, "strprop": "boo", "numprop2": 4};

	trace("//Array.prototype[2] = \"hole10\";");
	Array.prototype[2] = "hole10";

	trace("//Array.prototype[3] = \"hole11\";");
	Array.prototype[3] = "hole11";

	trace("//Array.prototype[4] = \"hole12\";");
	Array.prototype[4] = "hole12";

	trace("//(properties of contents of a)");
	assert_array_props(a);
	
	trace("//(cleaning up our holes...)");
	
	delete Array.prototype[2];
	delete Array.prototype[4];
	
	trace("//Array.prototype[3] = item4;");
	Array.prototype[3] = item4;
}

trace("//var item1 = {\"numprop\": 3, \"strprop\": \"Abc\", \"numprop2\": 3}");
var item1 = {"numprop": 5, "strprop": "Abc", "numprop2": 3};

trace("//var item2 = {\"numprop\": 3, \"strprop\": \"Azc\", \"numprop2\": 2}");
var item2 = {"numprop": 3, "strprop": "Azc", "numprop2": 2};

trace("//var item3 = {\"numprop\": 3, \"strprop\": \"aXc\", \"numprop2\": 1}");
var item3 = {"numprop": 7, "strprop": "aXc", "numprop2": 1};

trace("//var item4 = {\"numprop\": 3, \"strprop\": \"boo\", \"numprop2\": 4}");
var item4 = {"numprop": 9, "strprop": "boo", "numprop2": 4};

trace("//var item5 = {\"numprop\": 5, \"strprop\": \"bool\", \"numprop2\": \"5\"}");
var item5 = {"numprop": 11, "strprop": "bool", "numprop2": "5"};

var a = fresh_array_a();

trace("//Array.prototype[3] = item4;");
Array.prototype[3] = item4;

trace("//a.sortOn(\"numprop\", Array.UNIQUESORT) === 0");
trace(a.sortOn("numprop", Array.UNIQUESORT) === 0);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"]))");
assert_array_props(a.sortOn(["numprop", "strprop"]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.CASEINSENSITIVE | Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], Array.CASEINSENSITIVE))");
assert_array_props(a.sortOn(["numprop", "strprop"], Array.CASEINSENSITIVE));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.DESCENDING | Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], Array.DESCENDING))");
assert_array_props(a.sortOn(["numprop", "strprop"], Array.DESCENDING));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.CASEINSENSITIVE | Array.DESCENDING | Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], Array.CASEINSENSITIVE | Array.DESCENDING))");
assert_array_props(a.sortOn(["numprop", "strprop"], Array.CASEINSENSITIVE | Array.DESCENDING));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.NUMERIC | Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], Array.NUMERIC))");
assert_array_props(a.sortOn(["numprop", "strprop"], Array.NUMERIC));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], Array.DESCENDING | Array.NUMERIC | Array.RETURNINDEXEDARRAY))");
assert_array(a.sortOn(["numprop", "strprop"], Array.DESCENDING | Array.NUMERIC | Array.RETURNINDEXEDARRAY));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], Array.DESCENDING | Array.NUMERIC))");
assert_array_props(a.sortOn(["numprop", "strprop"], Array.DESCENDING | Array.NUMERIC));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY, 0]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY, 0]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [0, 0]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [0, 0]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY, Array.DESCENDING]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY, Array.DESCENDING]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [0, Array.DESCENDING]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [0, Array.DESCENDING]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY | Array.DESCENDING, 0]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY | Array.DESCENDING, 0]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [Array.DESCENDING, 0]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [Array.DESCENDING, 0]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY, Array.CASEINSENSITIVE]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY, Array.CASEINSENSITIVE]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [0, Array.CASEINSENSITIVE]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [0, Array.CASEINSENSITIVE]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE, 0]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE, 0]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [Array.CASEINSENSITIVE, 0]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [Array.CASEINSENSITIVE, 0]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY, Array.CASEINSENSITIVE | Array.DESCENDING]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY, Array.CASEINSENSITIVE | Array.DESCENDING]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [0, Array.CASEINSENSITIVE | Array.DESCENDING]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [0, Array.CASEINSENSITIVE | Array.DESCENDING]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE | Array.DESCENDING, 0]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE | Array.DESCENDING, 0]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [Array.CASEINSENSITIVE | Array.DESCENDING, 0]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [Array.CASEINSENSITIVE | Array.DESCENDING, 0]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY | Array.DESCENDING, Array.CASEINSENSITIVE]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY | Array.DESCENDING, Array.CASEINSENSITIVE]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [Array.DESCENDING, Array.CASEINSENSITIVE]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [Array.DESCENDING, Array.CASEINSENSITIVE]));
test_holes(a);

a = fresh_array_a();

trace("//(contents of a.sortOn([\"numprop\", \"strprop\"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE, Array.DESCENDING]))");
assert_array(a.sortOn(["numprop", "strprop"], [Array.RETURNINDEXEDARRAY | Array.CASEINSENSITIVE, Array.DESCENDING]));

trace("//(properties of contents of a.sortOn([\"numprop\", \"strprop\"], [Array.CASEINSENSITIVE, Array.DESCENDING]))");
assert_array_props(a.sortOn(["numprop", "strprop"], [Array.CASEINSENSITIVE, Array.DESCENDING]));
test_holes(a);

a = fresh_array_a();

trace("//a.sortOn([\"strprop\", \"numprop\"], [Array.NUMERIC, Array.UNIQUESORT]) === 0");
trace(a.sortOn(["strprop", "numprop"], [Array.NUMERIC, Array.UNIQUESORT]) === 0);