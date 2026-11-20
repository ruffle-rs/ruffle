// Check all possible ASnative "major" indices to see which
// ones exist ("minor" indices can't by checked that way).

for (var i = 0; i < 65536; i++) {
	var f = ASnative(i, 9999);
	if (f !== undefined) {
		var t = typeof f;
		trace("ASnative(" + i + ", _): " + t);
	}
}

trace("Done");
fscommand("quit");
