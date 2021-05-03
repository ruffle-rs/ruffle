package {
	public class Test {}
}

var t = true;
var f = false;

if (t) {
	trace("if (true)");
} else {
	trace("TEST FAIL: if (true)");
}

if (!f) {
	trace("if (!false)");
} else {
	trace("TEST FAIL: if (!false)");
}

if (!t) {
	trace("TEST FAIL: if (!true)");
} else {
	trace("if (!true)");
}

if (f) {
	trace("TEST FAIL: if (false)");
} else {
	trace("if (false)");
}