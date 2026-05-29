FIRST = 1
SECOND = 2
THIRD = 3.5
FOURTH = false
FIFTH = {}
SIXTH = null

function TEST_FIRST() {
	trace("Test_First");
}

function TEST_SECOND() {
	trace("Test_second");
}

function TEST_THIRD() {
	trace("Test_third");
}

function TEST_FOURTH() {
	trace("Test_bool");
}

function TEST_FIFTH() {
	trace("Test_obj");
}

function TEST_SIXTH() {
	trace("Test_null");
}

test = new Object();
test[FIRST] = TEST_FIRST;
test[SECOND] = TEST_SECOND;
test[THIRD] = TEST_THIRD;
test[FOURTH] = TEST_FOURTH;
test[FIFTH] = TEST_FIFTH;
test[SIXTH] = TEST_SIXTH;

test[FIRST]();
test[SECOND]();
test[THIRD]();
test[FOURTH]();
test[FIFTH]();
test[SIXTH]();

// Test empty/undefined method
var f = function() {
	trace("f");
}
var obj = {toString: function() { return ""; }};
f[undefined]();
f[""]();
f[obj]();

fscommand("quit");
