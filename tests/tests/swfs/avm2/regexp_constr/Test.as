package {
	public class Test {
	}
}

trace("// new RegExp();");
var re = new RegExp();
trace(re);
trace(re.source == "");
trace("dotall", re.dotall);
trace("extended", re.extended);
trace("global", re.global);
trace("ignoreCase", re.ignoreCase);
trace("multiline", re.multiline);
trace("");

function test(source:*, flags:*) {
    var sourceStr = (typeof source === "string") ? "\"" + source + "\"" : source;
    var flagsStr = (typeof flags === "string") ? "\"" + flags + "\"" : flags;
    trace("// new RegExp(" + sourceStr + ", " + flagsStr + ");");
	var re = new RegExp(source, flags);
	trace(re);
	trace(re.source == source);
	trace("dotall", re.dotall);
	trace("extended", re.extended);
	trace("global", re.global);
	trace("ignoreCase", re.ignoreCase);
	trace("multiline", re.multiline);
	trace("");
}

test("empty flags", "");
test("dotall flag", "s");
test("extended flag", "x");
test("global flag", "g");
test("ignoreCase flag", "i");
test("multiline flag", "m");
test("all flags", "sxgim");

test("invalid flags", "|%?-/.あa");
test("uppercase flags", "SXGIM");
test("duplicate flags", "ssgg");

test(undefined, undefined);
test(null, null);
test(/#((.*))$/m, undefined);
test(/empty flags/, undefined);
test(/dotall embedded flags/s, undefined);
try {
    test(/empty string separate flag/s, "");
} catch(e) {
    trace(e);
}
try {
    test(/dotall separate flags/s, "s");
} catch(e) {
    trace(e);
}
