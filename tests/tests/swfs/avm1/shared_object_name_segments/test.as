function show(name) {
    var so = SharedObject.getLocal(name);
    trace("\"" + name + "\": " + (so == null ? "null" : "ok"));
}

function same(a, b) {
    trace("\"" + a + "\" == \"" + b + "\": " + (SharedObject.getLocal(a) == SharedObject.getLocal(b)));
}

var names = [
    "save", ".save", "..save", "...", "dir/.save", "x/y/z",
    "", "/", "/save", "save/",
    ".", "..", "./save", "././save", "./a/b", "a/./b", "a/b/./c", "./.", "./..", "save/.", "save/..", "a/./.",
    "../save", "dir/../save", "a/b/../c", "a/./../b", "a//b"
];
for (var i = 0; i < names.length; i++) {
    show(names[i]);
}

same("save", "./save");
same("save", "././save");
same("a/b", "./a/b");
same("a/b", "a/./b");
same("a/b/c", "a/b/./c");
same(".", "./.");
same("..", "./..");
same("save", "/save");
same("save", "save/");
same(".save", "save");
