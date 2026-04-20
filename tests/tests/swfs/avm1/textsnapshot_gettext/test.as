function check(name, value) {
    var value2 = value;

    if (typeof value2 === "string") {
        value2 = value2.split("\r").join("\\r").split("\n").join("\\n");
    }

    trace(name + ": " + value2 + " [" + (typeof value) + "]");
}

var ts = new TextSnapshot(_root);

check("getCount()", ts.getCount());

check("getText()", ts.getText());
check("getText(0)", ts.getText(0));
check("getText(1)", ts.getText(1));
check("getText(1, 3)", ts.getText(1, 3));

var threeLike = {};
threeLike.valueOf = function() { return 3; }
check("getText(3-like, 5)", ts.getText(threeLike, 5));
check("getText(1, 3-like)", ts.getText(1, threeLike));
check("getText(\"3\", \"5\")", ts.getText(3, 5));

check("getText(100, 200)", ts.getText(100, 200));
check("getText(2, 200)", ts.getText(2, 200));
check("getText(200, 1)", ts.getText(200, 1));
check("getText(3, 1)", ts.getText(3, 1));

check("getText(100, 1)", ts.getText(100, 1));
check("getText(101, 1)", ts.getText(101, 1));
check("getText(100, 101)", ts.getText(100, 101));
check("getText(0, 0)", ts.getText(0, 0));
check("getText(1, 1)", ts.getText(1, 1));
check("getText(1, 0)", ts.getText(1, 0));

check("getText(true, false)", ts.getText(true, false));
check("getText(false, true)", ts.getText(false, true));

check("getText(0, 1, 2)", ts.getText(0, 1, 2));
check("getText(1, 10, 1)", ts.getText(1, 10, 1));
check("getText(1, 10, true)", ts.getText(1, 10, true));
check("getText(1, 10, false)", ts.getText(1, 10, false));
check("getText(1, 10, new Object())", ts.getText(1, 10, new Object()));
check("getText(1, 10, 0)", ts.getText(1, 10, 0));
check("getText(1, 10, 0/0)", ts.getText(1, 10, 0/0));

check("getText(8, 10, true)", ts.getText(8, 10, true));
check("getText(8, 10, false)", ts.getText(8, 10, false));

check("getText(0, 100)", ts.getText(0, 100));
check("getText(0, 100, true)", ts.getText(0, 100, true));
check("getText(0, 6)", ts.getText(0, 6));
check("getText(0, 6, true)", ts.getText(0, 6, true));
check("getText(0, 7)", ts.getText(0, 7));
check("getText(0, 7, true)", ts.getText(0, 7, true));

check("getText(1, 10, 1, 2)", ts.getText(1, 10, 1, 2));

check("getText(-5, -2)", ts.getText(-5, -2));
check("getText(-4, -2)", ts.getText(-4, -2));
check("getText(-3, -2)", ts.getText(-3, -2));
check("getText(-2, -2)", ts.getText(-2, -2));
check("getText(-1, -2)", ts.getText(-1, -2));
check("getText(0, -2)", ts.getText(0, -2));
check("getText(1, -2)", ts.getText(1, -2));
check("getText(2, -2)", ts.getText(2, -2));

check("getText(-5, -2)", ts.getText(-5, -2));
check("getText(-5, -1)", ts.getText(-5, -1));
check("getText(-5, 0)", ts.getText(-5, 0));
check("getText(-5, 1)", ts.getText(-5, 1));
check("getText(-5, 2)", ts.getText(-5, 2));
check("getText(-5, 3)", ts.getText(-5, 3));

check("getText(-5, 2)", ts.getText(-5, 2));
check("getText(5, -2)", ts.getText(5, -2));
check("getText(-2, -5)", ts.getText(-2, -5));
check("getText(-2, 5)", ts.getText(-2, 5));
check("getText(2, -5)", ts.getText(2, -5));
