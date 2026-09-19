// Frame 2 of the root timeline. `readAll` was defined by the child's frame 1.
// Once the child is removed, its variables resolve on the call's target instead:
// `this` if it's a clip, otherwise the caller's target. `_global` is still consulted last.
var shared = "root";
var rootOnly = "root";
_global.shared = "global";
_global.childOnly = "global";
_global.globalOnly = "global";

trace("// From root, before removal");
trace(readAll());

child.swapDepths(1000);
child.removeMovieClip();

trace("// From root, after removal");
trace(readAll());

trace("// From root, after removal, this = caller");
caller.readAllAsMethod = readAll;
trace(caller.readAllAsMethod());

trace("// From root, after removal, this = plain object");
var obj = {shared: "obj", objOnly: "obj"};
obj.readAllAsMethod = readAll;
trace(obj.readAllAsMethod());

trace("// From root, after removal, this = undefined (via call)");
trace(readAll.call(undefined));
