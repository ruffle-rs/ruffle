function keys(a) {
	var ks = [];
	for (var k in a) ks.push(k);
	return ks;
}

var a = [];
a[0] = "foo";
trace("// a.length");
trace(a.length);
trace("// a[0]");
trace(a[0]);

a.length = 0;
trace("// a.length");
trace(a.length);
trace("// a[0]");
trace(a[0]);

a.length = 1;
trace("// a.length");
trace(a.length);
trace("// a[0]");
trace(a[0]);

a[1] = "foo";
a[3] = "foo";
trace("// a.length");
trace(a.length);

a[-5] = "foo";
a[2147483648] = "foo"; // 2^31
trace("// a.length");
trace(a.length);

trace("// keys(a)");
trace(keys(a));

a[2147483647] = "foo"; // 2^31 - 1
trace("// a.length");
trace(a.length);
trace("// keys(a)");
trace(keys(a));

a[2147483648] = "foo"; // 2^31
trace("// a.length");
trace(a.length);

a[2147483649] = "foo"; // 2^31 + 1
trace("// a.length");
trace(a.length);

a[4294967294] = "foo"; // 2^32 - 2
trace("// a.length");
trace(a.length);

a[4294967295] = "foo"; // 2^32 - 1
trace("// a.length");
trace(a.length);

a[4294967296] = "foo"; // 2^32
trace("// a.length");
trace(a.length);

trace("// keys(a)");
trace(keys(a));

a[-2147483649] = "foo"; // -2^31 - 1
trace("// a.length");
trace(a.length);

a[-4294967296] = "foo"; // -2^32
trace("// a.length");
trace(a.length);

a["302231454903659441160191"] = "foo"; // 2^78 + 2^31 - 1
trace("// a.length");
trace(a.length);

a["302231454903657293676544"] = "foo"; // 2^78
trace("// a.length");
trace(a.length);

fscommand("quit");
