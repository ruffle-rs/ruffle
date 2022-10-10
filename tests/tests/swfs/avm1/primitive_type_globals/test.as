// Test that the primitive type globals (Number, Boolean, String) work as expected.
// They should return values when called as a function, and box them into objects
// when called a constructor

// Flash tends to compile raw Number(5) into a ToNumber opcode,
// so call the function indirectly to avoid this.
var num = Number;
trace('// Number');
trace(num);
trace('');

trace('// Number()');
var n = num();
trace(typeof n);
trace(n);
trace('');

trace('// Number(1)');
var n = num(1);
trace(typeof n);
trace(n)
trace('');

trace('// new Number("-101")');
var n = new Number("-101");
trace(typeof n);
trace(n)
trace('');

trace('// (5).valueOf()');
trace((5).valueOf());
trace('');

trace('// typeof (5).valueOf()');
trace(typeof (5).valueOf());
trace('');

trace('// (5).toString()');
trace((5).toString());
trace('');

trace('// typeof (5).toString()');
trace(typeof (5).toString());
trace('');

for (var i = 0; i < 38; i++){
    trace('// (4123).toString(' + i + ')');
    trace((4123).toString(i));
    trace('');
}

for (var i = 0; i < 38; i++){
    trace('// (-123.1).toString(' + i + ')');
    trace((-123.1).toString(i));
    trace('');
}

for (var i = 0; i < 38; i++){
    trace('// (-2147483647.9).toString(' + i + ')');
    trace((-2147483647.9).toString(i));
    trace('');
}

for (var i = 0; i < 38; i++){
    trace('// NaN.toString(' + i + ')');
    trace(NaN.toString(i));
    trace('');
}

trace('// Number.NaN');
trace(num.NaN);
trace('');

trace('// Number.POSITIVE_INFINITY');
trace(num.POSITIVE_INFINITY);
trace('');

trace('// Number.NEGATIVE_INFINITY');
trace(num.NEGATIVE_INFINITY);
trace('');

trace('// Number.MIN_VALUE');
trace(num.MIN_VALUE);
trace('');

trace('// Number.MAX_VALUE');
trace(num.MAX_VALUE);
trace('');

var f = Boolean;

trace('// Boolean');
trace(f);
trace('');

trace('// Boolean()');
var b = f();
trace(b);
trace(typeof b);
trace('');

trace('// Boolean(false)');
var b = f(false);
trace(b);
trace(typeof b);
trace('');

trace('// Boolean("asd")');
var b = f("asd");
trace(b);
trace(typeof b);
trace('');

trace('// new Boolean()');
var b = new Boolean();
trace(b);
trace(typeof b);
trace('');

trace('// new Boolean(true)');
var b = new Boolean(true);
trace(b);
trace(typeof b);
trace('');

trace('// new Boolean(1)');
var b = new Boolean(1);
trace(b);
trace(typeof b);
trace('');

trace('// new Boolean("ASD")');
var b = new Boolean("ASD");
trace(b);
trace(typeof b);
trace('');

trace('// b.toString()');
var b = true;
trace(b.toString());
trace('');

trace('// b.valueOf()');
var b = true;
trace(b.valueOf());
trace('');

var f = String;

trace('// String');
trace(f);
trace('');

trace('// String()');
var s = f();
trace(s);
trace(typeof s);
trace('');

trace('// String("foo")');
var s = f("foo");
trace(s);
trace(typeof s);
trace('');

trace('// new String("333")');
var s = new String("333");
trace(s);
trace(typeof s);
trace('');

trace('// s.toString()');
var s = "test";
trace(s.toString());
trace('');

trace('// s.valueOf()');
var s = "test2";
trace(s.valueOf());
trace('');
