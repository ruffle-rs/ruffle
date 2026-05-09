trace("// int(undefined)");
trace(int(undefined));
trace("// int(null)");
trace(int(null));
trace("// int(false)");
trace(int(false));
trace("// int(0)");
trace(int(0));
trace("// int('test')");
trace(int('test'));
trace("// int('foobarbaz')");
trace(int("foobarbaz"));
trace("// int({})");
trace(int({}));

var hello = "hello";
trace("// int(hello)");
trace(int(hello));

var ten = "10";
trace("// int(ten)");
trace(int(ten));

var float = "10.5";
trace("// int(float)");
trace(int(float));

var negative = "-10";
trace("// int(negative)");
trace(int(negative));

var nan = NaN;
trace("// int(NaN)");
trace(int(nan));

var valof = {};
valof.valueOf = function() {
	return 42;
}
trace("// int(valof)");
trace(int(valof));

trace("// int(0xffffffff)");
trace(int(0xffffffff));

fscommand("quit");
