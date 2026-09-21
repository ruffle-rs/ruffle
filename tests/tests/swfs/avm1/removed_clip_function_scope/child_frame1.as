// Frame 1 of the `child` sprite.
var shared = "child";
var childOnly = "child";

function readAll() {
    return shared + " " + childOnly + " " + rootOnly + " " + callerOnly + " " + globalOnly;
}

_root.readAll = readAll;
