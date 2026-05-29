// Sort normal (strings).
var a = ["d", "c", "a", "b"];
trace('// ["d", "c", "a", "b"].sort()');
trace(a.sort() == a);
trace(a);
trace("");

// Bogus parameters.
trace('// [1, 2].sort(undefined)');
trace([1, 2].sort(undefined));
trace("");

// Bogus parameters.
trace('// [1, 2].sort(true)');
trace([1, 2].sort(true));
trace("");

// Bogus parameters.
trace('// [1, 2].sort(true, 0)');
trace([1, 2].sort(true, 0));
trace("");


// Bogus parameters.
trace('// [1, 2].sort(undefined, 0)');
trace([1, 2].sort(undefined, 0));
trace("");

// Bogus parameters.
trace('// [1, 2].sort(null, 0)');
trace([1, 2].sort(null, 0));
trace("");

// Works.
trace('// [1, 2].sort(NaN)');
trace([1, 2].sort(NaN));
trace("");

// Sort normal (weirdos).
var o = {toString: function() { return "AAA"; }};
trace('// [undefined, null, true, false, o].sort()');
var a = [undefined, null, true, false, o];
trace(a.sort());
trace("");

// Sort descending.
var a = ["d", "c", "a", "b"];
trace('// ["d", "c", "a", "b"].sort(Array.DESCENDING)');
trace(a.sort(Array.DESCENDING));
trace("");

// Sort normal (numbers, but sorts based on strings).
var a = [4, 1, 3, 22, 2, 3];
trace('// [4, 1, 3, 22, 2, 3].sort()');
trace(a.sort());
trace("");

// Sort numeric.
var a = [4, 1, 3, 22, 2, 3];
trace('// [4, 1, 3, 22, 2, 3].sort(Array.NUMERIC)');
trace(a.sort(Array.NUMERIC));
trace("");

// Sort numeric with bad values.
var a = [Infinity, NaN, 4, 1, NaN, -Infinity];
trace('// [Infinity, NaN, 4, 1, NaN, -Infinity].sort(Array.NUMERIC)');
trace(a.sort(Array.NUMERIC));
trace("");

// Sort descending + numeric.
var a = [4, 1, 3, 22, 2, 3];
trace('// [4, 1, 3, 22, 2, 3].sort(Array.DESCENDING | Array.NUMERIC)');
trace(a.sort(Array.DESCENDING | Array.NUMERIC));
trace("");

// Sort unique.
var a = [4, 1, 3, 22, 2];
trace('// [4, 1, 3, 22, 2].sort(Array.UNIQUESORT)');
trace(a.sort(Array.UNIQUESORT));
trace(a);
trace("");

// Sort unique.
var a = [4, 1, 3, 22, 2, 3];
trace('// [4, 1, 3, 22, 2, 3].sort(Array.UNIQUESORT)');
trace(a.sort(Array.UNIQUESORT)); // returns 0
trace(a); // Should not modify original array.
trace("");

// Sort case insensitive.
var a = ["hëllo", "HËLLO", "TeSt", "test"];
trace('// ["hëllo", "HËLLO", "TeSt", "test"].sort(Array.CASEINSENSITIVE)');
trace(a.sort(Array.CASEINSENSITIVE));
trace("");

var a = ["TeSt", "hëllo", "HËLLO", "test"];
trace('// ["TeSt", "hëllo", "HËLLO", "test"].sort(Array.CASEINSENSITIVE)');
trace(a.sort(Array.CASEINSENSITIVE));
trace(a);
trace("");

// Sort case insensitive and unique.
var a = ["TeSt", "hëllo", "HËLLO", "test"];
trace('// ["TeSt", "hëllo", "HËLLO", "test"].sort(Array.CASEINSENSITIVE | Array.UNIQUESORT)');
trace(a.sort(Array.CASEINSENSITIVE | Array.UNIQUESORT));
trace(a);
trace("");

// Return indexed array.
var a = ["d", "c", "a", "b"];
trace('// ["d", "c", "a", "b"].sort(Array.RETURNINDEXEDARRAY)');
trace(a.sort(Array.RETURNINDEXEDARRAY));
trace(a);
trace("");

// The works
var a = ["test", 4, 1, "22", undefined, 3, 23, 2, true];
trace('// ["test", 4, 1, "22", undefined, 3, 23, 2, true].sort(Array.NUMERIC | Array.CASEINSENSITIVE | Array.DESCENDING | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY)');
trace(a.sort(Array.NUMERIC | Array.CASEINSENSITIVE | Array.DESCENDING | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));
trace(a);
trace("");

// Custom compare function.
function mySort(a, b) {
	if( a.n < b.n ) {
		return -1;
	} else if( a.n > b.n) {
		return 1;
	} else {
		return 0;
	}
}

function traceArray(a) {
	function printObj(o) {
		if( typeof o == "object" ) {
			var out = [];
			for( var k in o ) {
				out.push(k + ": " + o[k]);
			}

			var s = "{";
			s += out.join(", ");
			s += "}";
			return s;
		} else {
			return o.toString();
		}
	}

	var s = printObj(a[0]);
	for(var i=1; i<a.length; i++)
	{
		s += "," + printObj(a[i]);
	}
	trace(s);
}

var a = [{n: 3}, {n: 5}, {n: 1}, {n: 2}, {n: 1}];
trace('// [objects..].sort(mySort)');
traceArray(a.sort(mySort));
trace("");

var a = [{n: 3}, {n: 5}, {n: 1}, {n: 2}, {n: 1}];
trace('// [objects..].sort(mySort, Array.DESCENDING)');
traceArray(a.sort(mySort, Array.DESCENDING));
trace("");

var a = [{n: 3}, {n: 5}, {n: 1}, {n: 2}, {n: 1}];
trace('// [objects..].sort(mySort, Array.UNIQUESORT)');
trace(a.sort(mySort, Array.UNIQUESORT));
traceArray(a);
trace("");

// Not a function.
var a = [{n: 3}, {n: 5}, {n: 1}, {n: 2}, {n: 1}];
trace('// [objects..].sort({})');
trace(a.sort({}));
traceArray(a);
trace("");

// Not a function.
var a = [{n: 3}, {n: 5}, {n: 1}, {n: 2}, {n: 1}];
trace('// [objects..].sort({}, undefined)');
trace(a.sort({}, undefined));
trace("");

// Not a function.
var a = [2, 3, 1];
trace('// [2, 3, 1].sort(55, undefined)');
trace(a.sort(55, undefined));
trace("");

// Not a function, flags used twice. Second flags is what is used, normal compare function.
var a = [2, 3, 1];
trace('// [2, 3, 1].sort(Array.DESCENDING, Array.NUMERIC)');
trace(a.sort(Array.DESCENDING, Array.NUMERIC));
trace("");


// sortOn tests

// Normal
var a = [{n: 3}, {n: 5}, {n: 22}, {n: 2}, {n: 1}];
trace('// [objects..].sortOn("n")');
trace(a.sortOn("n") == a);
traceArray(a);
trace("");

// Descending
var a = [{n: 3}, {n: 5}, {n: 22}, {n: 2}, {n: 1}];
trace('// [objects..].sortOn("n", Array.DESCENDING)');
traceArray(a.sortOn("n", Array.DESCENDING));
trace("");

// Numeric
var a = [{n: 3}, {n: 5}, {n: 22}, {n: 2}, {n: 1}];
trace('// [objects..].sortOn("n", Array.NUMERIC)');
traceArray(a.sortOn("n", Array.NUMERIC));
trace("");

// Case insensitive
var a = [{n: "hello"}, {n: "HELLO"}, {n: "test"}, {n: "TEST"}];
trace('// [objects..].sortOn("n", "n", Array.CASEINSENSITIVE)');
traceArray(a.sortOn("n", Array.CASEINSENSITIVE));
trace("");

// Unique
var a = [{n: "test"}, {n: "hello"}, {n: "TEST"}, {n: "hello"}];
trace('// [objects..].sortOn("n", Array.UNIQUESORT)');
trace(a.sortOn("n", Array.UNIQUESORT));
traceArray(a);
trace("");

// Return idnex array
var a = [{n: 3}, {n: 5}, {n: 22}, {n: 2}, {n: 1}];
trace('// [objects..].sortOn("n", Array.UNIQUESORT)');
trace(a.sortOn("n", Array.RETURNINDEXEDARRAY));
traceArray(a);
trace("");

// The works
var a = [{n: 3}, {n: 5}, {n: 22}, {n: 2}, {n: 1}];
trace('// [objects..].sortOn("n", Array.NUMERIC | Array.CASEINSENSITIVE | Array.DESCENDING | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY)');
trace(a.sortOn("n", Array.NUMERIC | Array.CASEINSENSITIVE | Array.DESCENDING | Array.UNIQUESORT | Array.RETURNINDEXEDARRAY));
traceArray(a);
trace("");

// Primitive types-- autoboxing doesn't happen??
var a = ["test", "asdasdasdsad", "bar", "hello", "a"];
trace('// [strings..].sortOn("length", Array.NUMERIC)');
trace(a.sortOn("laength", Array.NUMERIC));
trace("");

// Manually boxed strings.
var a = [new String("hello"), new String("test"), new String("a"), new String("asdasdasdsad"), new String("bar")];
trace('// [stringobjs..].sortOn("length", Array.NUMERIC)');
trace(a.sortOn("length", Array.NUMERIC));
trace("");

// Multiple fields
var a = [{n: 3, b: 1}, {n: 2, b: 3}, {n: 2, b: 2}, {n: 1, b: 2}];
trace('// [objects..].sortOn(["n", "b"])');
traceArray(a.sortOn(["n", "b"]));
trace("");

// Multiple fields and flags!!
var a = [{n: "foo", b: 1}, {n: "bar", b: 3}, {n: "BAR", b: 22}, {n: "foo", b: 2}];
trace('// [objects..].sortOn(["n", "b"], [Array.CASEINSENSITIVE, Array.NUMERIC])');
traceArray(a.sortOn(["n", "b"], [Array.CASEINSENSITIVE, Array.NUMERIC]));
trace("");

// Not enough flags! (flags ignored)
var a = [{n: "foo", b: 1}, {n: "bar", b: 3}, {n: "BAR", b: 22}, {n: "foo", b: 2}];
trace('// [objects..].sortOn(["n", "b"], [Array.DESCENDING])');
traceArray(a.sortOn(["n", "b"], [Array.DESCENDING]));
trace("");

// Too many flags! (flags ignored)
var a = [{n: "foo", b: 1}, {n: "bar", b: 3}, {n: "BAR", b: 22}, {n: "foo", b: 2}];
trace('// [objects..].sortOn(["n", "b"], [Array.DESCENDING, Array.0, 0])');
traceArray(a.sortOn(["n", "b"], [Array.DESCENDING, 0, 0]));
trace("");

// Unique
var a = [{n: 3, b: 1}, {n: 2, b: 3}, {n: 2, b: 2}, {n: 1, b: 2}, {n: 3, b: 1}];
trace('// [objects..].sortOn(["n", "b"], [Array.UNIQUESORT, Array.0])');
trace(a.sortOn(["n", "b"], [Array.UNIQUESORT, 0]));
traceArray(a);
trace("");

// Return indexed
var a = [{n: 3, b: 1}, {n: 2, b: 3}, {n: 2, b: 2}, {n: 1, b: 2}, {n: 3, b: 1}];
trace('// [objects..].sortOn(["n", "b"], [Array.RETURNINDEXEDARRAY, 0])');
trace(a.sortOn(["n", "b"], [Array.RETURNINDEXEDARRAY, 0]));
traceArray(a);
trace("");

// Unqiue/return indexed only applies to first set of flags
var a = [{n: 3, b: 1}, {n: 2, b: 3}, {n: 2, b: 2}, {n: 1, b: 2}, {n: 3, b: 1}];
trace('// [objects..].sortOn(["n", "b"], [0, Array.RETURNINDEXEDARRAY | Array.UNIQUESORT])');
trace(a.sortOn(["n", "b"], [0, Array.RETURNINDEXEDARRAY | Array.UNIQUESORT]));
traceArray(a);
trace("");

// Empty fields.
trace('// [1, 2].sortOn([])');
trace([1, 2].sortOn([]));
trace("");

// Bogus params.
trace('// [1, 2].sortOn()');
trace([1, 2].sortOn());
trace("");

// Coerces to "undefined"
trace('// [1, 2].sortOn(undefined)');
trace([1, 2].sortOn(undefined));
trace("");

trace("// sortOn w/ __proto__ field");

var o1 = {name: "o1"};
o1.__proto__ = {order: 1};

var o2 = {name: "o2"};
o2.__proto__ = {order: 2};

var arr = [o2, o1];
arr.sortOn("order", Array.NUMERIC); // doesn't trace any of the above

for(var i = 0; i < arr.length; i++)
{
    trace(arr[i].name);
}
trace("");


trace("// sortOn w/ getter");

var o1 = {name: "o1"};
o1.addProperty("order", function() { trace("O1 order"); return 1; }, null);

var o2 = {name: "o2"};
o2.addProperty("order", function() { trace("O2 order"); return 2; }, null);

var arr = [o2, o1];
arr.sortOn("order", Array.NUMERIC); // doesn't trace any of the above

for(var i = 0; i < arr.length; i++)
{
    trace(arr[i].name);
}
trace("");

fscommand("quit");
