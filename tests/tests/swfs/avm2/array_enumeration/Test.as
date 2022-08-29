package {
	public class Test {
	}
}

var a = [1, 2, 3, 4, 5];
for each(var i in a) {
    trace(i)
}

trace("Enumerating with holes");

var array = new Array(5);
array[0] = "elem0";
array[4] = "elem4";
array.prop = "property";
array[-1] = "elem negative one";

var indices = [];
for (var i in array) {
	indices.push(i);
}

var entries = [];
for each(var entry in array) {
	entries.push(entry);
}

// FIXME - the enumeration order of local properties
// doesn't currently match between Ruffle and Flash Player,
// since the Flash Player order depends on the hash of the ScriptObjects
// stored in the internal hash table.
// For now, we just assert that we see the expected number of properties, and that
// the 'normal' array entries come first

trace("Indices len: ", indices.length);
trace("Entires len: ", entries.length);
trace("First indices: ", indices[0], indices[1]);
trace("First entires: ", entries[0], entries[1]);