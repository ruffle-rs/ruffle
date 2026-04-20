function check(name, value) {
    var value2 = value;

    if (typeof value2 === "string") {
        value2 = value2.split("\r").join("\\r").split("\n").join("\\n");
    }

    trace(name + ": " + value2 + " [" + (typeof value) + "]");
}

var ts = new TextSnapshot(_root);

check("getCount()", ts.getCount());

check("findText()", ts.findText());
check("findText(0)", ts.findText(0));
check("findText(1)", ts.findText(1));
check("findText(1, \"A\")", ts.findText(1, "A"));
check("findText(1, \"A\", true)", ts.findText(1, "A", true));
check("findText(1, \"A\", true, 1)", ts.findText(1, "A", true, 1));

check("findText(0, \"A\", true)", ts.findText(0, "A", true));
check("findText(0, \"A\", false)", ts.findText(0, "A", false));
check("findText(0, \"a\", false)", ts.findText(0, "a", false));
check("findText(0, \"a\", true)", ts.findText(0, "a", true));

check("findText(0, \"EF\", true)", ts.findText(0, "EF", true));
check("findText(0, \"eF\", true)", ts.findText(0, "eF", true));
check("findText(0, \"eF\", false)", ts.findText(0, "eF", false));

check("findText(1, \"EF\", true)", ts.findText(1, "EF", true));
check("findText(3, \"EF\", true)", ts.findText(3, "EF", true));
check("findText(4, \"EF\", true)", ts.findText(4, "EF", true));
check("findText(5, \"EF\", true)", ts.findText(5, "EF", true));

check("findText(0, \"FG\", true)", ts.findText(0, "FG", true));
check("findText(0, \"FG\", false)", ts.findText(0, "FG", false));
check("findText(0, \"gh\", true)", ts.findText(0, "gh", true));
check("findText(0, \"gh\", false)", ts.findText(0, "gh", false));
check("findText(0, \"F\\nG\", true)", ts.findText(0, "F\nG", true));
check("findText(0, \"F\\nG\", false)", ts.findText(0, "F\nG", false));

check("findText(0, \" \", false)", ts.findText(0, " ", false));

var threeLike = {};
threeLike.valueOf = function() { return 3; }
check("findText(3-like, \"A\", true)", ts.findText(threeLike, "A", true));
check("findText(3-like, \"E\", true)", ts.findText(threeLike, "E", true));

var stringLike = {};
stringLike.toString = function() { return "C"; }
check("findText(0, C-like, true)", ts.findText(0, stringLike, true));
check("findText(4, C-like, true)", ts.findText(4, stringLike, true));

check("findText(0, \"A\", 1)", ts.findText(0, "A", 1));
check("findText(0, \"A\", new Object())", ts.findText(0, "A", new Object()));
check("findText(0, \"A\", \"0\")", ts.findText(0, "A", "0"));

check("findText(0, \"ABCDEFGHIJK\", true)", ts.findText(0, "ABCDEFGHIJK", true));
check("findText(0, \"aBcdEfGHIjK\", true)", ts.findText(0, "aBcdEfGHIjK", true));
check("findText(0, \"aBcdEfGHIjK\", false)", ts.findText(0, "aBcdEfGHIjK", false));

check("findText(-1, \"A\", 1)", ts.findText(-1, "A", 1));
check("findText(-10, \"A\", 1)", ts.findText(-10, "A", 1));
check("findText(100, \"A\", 1)", ts.findText(100, "A", 1));

check("findText(0, \"\", true)", ts.findText(0, "", true));
check("findText(0, \"\", false)", ts.findText(0, "", false));
check("findText(1, \"\", true)", ts.findText(1, "", true));
check("findText(1, \"\", false)", ts.findText(1, "", false));
check("findText(-1, \"\", true)", ts.findText(-1, "", true));
check("findText(-1, \"\", false)", ts.findText(-1, "", false));
