package {
	public class Test {}
}

// Constants
trace("Math.E =", Math.E);
trace("Math.LN10 =", Math.LN10);
trace("Math.LN2 =", Math.LN2);
trace("Math.LOG10E =", Math.LOG10E);
trace("Math.LOG2E =", Math.LOG2E);
trace("Math.PI =", Math.PI);
trace("Math.SQRT1_2 =", Math.SQRT1_2);
trace("Math.SQRT2 =", Math.SQRT2);
trace();

var obj = {valueOf: function():Number { return 10.1; }};

function runTest(name, func, val) {
	trace(name + "(" + val + ") =");
	trace(func(val));
}

function test(name, func) {
	runTest(name, func, 0);
	runTest(name, func, 1);
	runTest(name, func, -1);
	runTest(name, func, 1234.5);
	runTest(name, func, -1234.5);
	runTest(name, func, Infinity);
	runTest(name, func, -Infinity);
	runTest(name, func, NaN);
	runTest(name, func, true);
	runTest(name, func, false);
	runTest(name, func, undefined);
	runTest(name, func, null);
	runTest(name, func, "55.5");
	runTest(name, func, obj);
	trace();
}

function runTest2(name, func, val1, val2) {
	trace(name + "(" + val1 + ", " + val2 + ") =");
	trace(func(val1, val2));
}

function test2(name, func) {
	runTest2(name, func, 0, 0);
	runTest2(name, func, 1, 2);
	runTest2(name, func, 2, -4);
	runTest2(name, func, 4, -2);
	runTest2(name, func, -99, -100);
	runTest2(name, func, Infinity, -Infinity);
	runTest2(name, func, NaN, 100);
	runTest2(name, func, 999, NaN);
	runTest2(name, func, true, false);
	runTest2(name, func, undefined, null);
	runTest2(name, func, "55.5", "-1234");
	runTest2(name, func, obj, obj);
	trace();
}

test("Math.abs", Math.abs);
test("Math.acos", Math.acos);
test("Math.asin", Math.asin);
test("Math.atan", Math.atan);
test2("Math.atan2", Math.atan2);
test("Math.ceil", Math.ceil);
test("Math.cos", Math.cos);
test("Math.exp", Math.exp);
test("Math.floor", Math.floor);
test("Math.log", Math.log);
test2("Math.max", Math.max);
test2("Math.min", Math.min);
test2("Math.pow", Math.pow);
test("Math.round", Math.round);
test("Math.sin", Math.sin);
test("Math.sqrt", Math.sqrt);
test("Math.tan", Math.tan);

// Test varargs in min/max
trace("Math.min() =", Math.min());
trace("Math.min(0) =", Math.min(0));
trace("Math.min(1, 2, 3) =", Math.min(1, 2, 3));
trace("Math.min(-1.1, -2.2, -3.3) =", Math.min(-1.1, -2.2, -3.3));
trace("Math.min(9, NaN, false, true, Infinity, undefined) =", Math.min(9, NaN, false, true, Infinity, undefined));
trace();

trace("Math.max() =", Math.max());
trace("Math.max(0) =", Math.max(0));
trace("Math.max(1, 2, 3) =", Math.max(1, 2, 3));
trace("Math.max(-1.1, -2.2, -3.3) =", Math.max(-1.1, -2.2, -3.3));
trace("Math.max(9, NaN, false, true, Infinity, undefined) =", Math.max(9, NaN, false, true, Infinity, undefined));
trace();

