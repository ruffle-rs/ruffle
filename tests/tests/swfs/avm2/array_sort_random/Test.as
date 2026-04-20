package {
	public class Test {
	}
}

// A simple deterministic PRNG; namely, Xorshift.
var rngState: int = 0x12345678;
function rng(): int {
	rngState ^= rngState << 13;
	rngState ^= rngState >>> 17;
	rngState ^= rngState << 5;
	return rngState;
}

var array: Array = new Array();
for (var i: int = 0; i < 50; i++) {
	array.push(i);
}

// "sort" the array using randomly-chosen comparison results.
array.sort(function(a:*, b:*): int {
	var r: int = rng();
	if (r % 8 == 0) {
		trace("cmp: " + a + " == " + b);
		return 0;
	} else if (r > 0) {
		trace("cmp: " + a + " > " + b);
		return 1;
	} else {
		trace("cmp: " + a + " < " + b);
		return -1;
	}
});

trace("// contents of array");
trace(array);
