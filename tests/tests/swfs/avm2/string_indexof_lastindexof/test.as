package {
	public class test {}
}

var s = "aaatestFOOtestaaanull";
trace("// // indexOf ");
// indexOf 
trace("// s.indexOf(\"a\")");
s.indexOf("a")
trace("// s.indexOf(\"a\", 16)");
s.indexOf("a", 16)
trace("// s.indexOf(\"a\", 14)");
s.indexOf("a", 14)
trace("// s.indexOf(\"a\", 13)");
s.indexOf("a", 13)
trace("// s.indexOf(\"a\", 0)");
s.indexOf("a", 0)
trace("// s.indexOf(\"test\")");
s.indexOf("test")
trace("// s.indexOf(\"test\", 4)");
s.indexOf("test", 4)
trace("// s.indexOf(\"test\", 100)");
s.indexOf("test", 100)
trace("// s.indexOf(\"test\", -1)");
s.indexOf("test", -1)
trace("// s.indexOf(\"test\", 4294967300)");
s.indexOf("test", 4294967300)
trace("// s.indexOf(\"test\", null)");
s.indexOf("test", null)
trace("// s.indexOf(\"test\", undefined)");
s.indexOf("test", undefined)
trace("// s.indexOf(\"\")");
s.indexOf("")
trace("// s.indexOf(\"\", 5)");
s.indexOf("", 5)
trace("// s.indexOf(\"\", 100)");
s.indexOf("", 100)
trace("// s.indexOf()");
s.indexOf()
trace("// s.indexOf(null)");
s.indexOf(null)
trace("// s.indexOf(undefined)");
s.indexOf(undefined)
trace("// \"hello undefined hi\".indexOf(undefined)");
"hello undefined hi".indexOf(undefined)

