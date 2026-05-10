trace("// original = [\"a\", \"b\", \"c\", \"d\", \"e\"]");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice()");
splice = original.splice();

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(undefined)");
splice = original.splice(undefined);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(0)");
splice = original.splice(0);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(null)");
splice = original.splice(null);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(2)");
splice = original.splice(2);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(6)");
splice = original.splice(6);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(2, 2)");
splice = original.splice(2, 2);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(2, undefined)");
splice = original.splice(2, undefined);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(2, 8)");
splice = original.splice(2, 8);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(1, 3, \"deleted\")");
splice = original.splice(1, 3, "deleted");

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(1, 2, \"x\", \"y\", \"z\")");
splice = original.splice(1, 2, "x", "y", "z");

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(0, 0, \"w\", [\"x\", \"y\"], \"z\")");
splice = original.splice(0, 0, "w", ["x", "y"], "z");

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

trace("");

original = ["a", "b", "c", "d", "e"];
trace("// splice = original.splice(2, -2)");
splice = original.splice(2, -2);

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("");

original = [];
trace("// splice = original.splice(1, 0, 'a')");
splice = original.splice(1, 0, 'a');

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("")


original = ['a', 'b'];
trace("// splice = original.splice(-9, 0, 'c')");
splice = original.splice(-9, 0, 'c');

trace("// original");
trace(original);
trace("");

trace("// original.length");
trace(original.length);
trace("");

trace("// splice");
trace(splice);
trace("");

trace("// splice.length");
trace(splice.length);
trace("")

trace("");

fscommand("quit");
