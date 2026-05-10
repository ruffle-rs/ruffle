original = ["a", "b", "c", "d", "e"];
duplicate = original.slice();
tail = original.slice(3);
middle = original.slice(1, 3);
head = original.slice(0, -3);
empty = original.slice(3, 2);
end = original.slice(-2, 4);

trace("// original");
trace(original);
trace("");

trace("// original.slice()");
trace(duplicate);
trace("");

trace("// original.slice(3)");
trace(tail);
trace("");

trace("// original.slice(1, 3)");
trace(middle);
trace("");

trace("// original.slice(0, -3)");
trace(head);
trace("");

trace("// original.slice(3, 2)");
trace(empty);
trace("");

trace("// original.slice(-2, 4)");
trace(end);
trace("");

var array = [1, 2, 3, 4, 5];
trace("// [1,2,3,4,5].slice(1)");
trace([1,2,3,4,5].slice(1));
trace("");

trace("// [1,2,3,4,5].slice(1, \"3\")");
var v = "3";
trace([1,2,3,4,5].slice(1, v));
trace("");

trace("// [1,2,3,4,5].slice(1, undefined)");
trace([1,2,3,4,5].slice(1, undefined));
trace("");

trace("// [1,2,3,4,5].slice(1, null)");
trace([1,2,3,4,5].slice(1, null));
trace("");

trace("// [1,2,3,4,5].slice(1, 0)");
trace([1,2,3,4,5].slice(1, 0));
trace("");

fscommand("quit");
