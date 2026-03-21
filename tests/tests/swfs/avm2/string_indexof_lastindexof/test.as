package {
	public class test {}
}

trace("// var s = \"aaatestFOOtestaaanull\"");
var s = "aaatestFOOtestaaanull";
trace("// // indexOf");
// indexOf
trace("// s.indexOf(\"a\")");
trace(s.indexOf("a"));
trace("// s.indexOf(\"a\", 16)");
trace(s.indexOf("a", 16));
trace("// s.indexOf(\"a\", 14)");
trace(s.indexOf("a", 14));
trace("// s.indexOf(\"a\", 13)");
trace(s.indexOf("a", 13));
trace("// s.indexOf(\"a\", 0)");
trace(s.indexOf("a", 0));
trace("// s.indexOf(\"test\")");
trace(s.indexOf("test"));
trace("// s.indexOf(\"test\", 4)");
trace(s.indexOf("test", 4));
trace("// s.indexOf(\"test\", 100)");
trace(s.indexOf("test", 100));
trace("// s.indexOf(\"test\", -1)");
trace(s.indexOf("test", -1));
trace("// s.indexOf(\"test\", 4294967300)");
trace(s.indexOf("test", 4294967300));
trace("// s.indexOf(\"test\", null)");
trace(s.indexOf("test", null));
trace("// s.indexOf(\"test\", undefined)");
trace(s.indexOf("test", undefined));
trace("// s.indexOf(\"\")");
trace(s.indexOf(""));
trace("// s.indexOf(\"\", 5)");
trace(s.indexOf("", 5));
trace("// s.indexOf(\"\", 100)");
trace(s.indexOf("", 100));
trace("// s.indexOf()");
trace(s.indexOf());
trace("// s.indexOf(null)");
trace(s.indexOf(null));
trace("// s.indexOf(undefined)");
trace(s.indexOf(undefined));
trace("// \"hello undefined hi\".indexOf(undefined)");
trace("hello undefined hi".indexOf(undefined));

