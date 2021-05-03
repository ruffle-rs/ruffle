package {
	public class Test {}
}

var t = true;
var f = false;

if (t === t) {
	trace("if (true === true)");
} else {
	trace("TEST FAIL: if (true === true)");
}

if (t === f) {
	trace("TEST FAIL: if (true === false)");
} else {
	trace("if (true === false)");
}

if (f === t) {
	trace("TEST FAIL: if (false === true)");
} else {
	trace("if (false === true)");
}

if (f === f) {
	trace("if (false === false)");
} else {
	trace("TEST FAIL: if (false === false)");
}

if (t !== t) {
	trace("TEST FAIL: if (true !== true)");
} else {
	trace("if (true !== true)");
}

if (t !== f) {
	trace("if (true !== false)");
} else {
	trace("TEST FAIL: if (true !== false)");
}

if (f !== t) {
	trace("if (false !== true)");
} else {
	trace("TEST FAIL: if (false !== true)");
}

if (f !== f) {
	trace("TEST FAIL: if (false !== false)");
} else {
	trace("if (false !== false)");
}