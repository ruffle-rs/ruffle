package {
	public class Test {
	}
}

trace('// new RegExp("").test("");');
trace(new RegExp("").test(""));

trace('// new RegExp("").test("abc");');
trace(new RegExp("").test("abc"));

trace('// /\d+/.test("abc");');
trace(/\d+/.test("abc"));

trace('// /\d+/.test("abc 123");');
trace(/\d+/.test("abc 123"));

trace('// /ABC/.test("abc");');
trace(/ABC/.test("abc"));

trace('// /ABC/i.test("abc");');
trace(/ABC/i.test("abc"));

trace('// /a.b/.test("a\\nb");');
trace(/a.b/.test("a\nb"));

trace('// /a.b/s.test("a\\nb");');
trace(/a.b/s.test("a\nb"));

trace('// /^bar/.test("foo\\nbar");');
trace(/^bar/.test("foo\nbar"));

trace('// /^bar/m.test("foo\\nbar");');
trace(/^bar/m.test("foo\nbar"));

// global flag
trace('// var re = new RegExp("[0-9]{3}", "g");');
var re = new RegExp("[0-9]{3}", "g");
trace('// re.lastIndex;');
trace(re.lastIndex);
trace('// re.test("0123456789");');
trace(re.test("0123456789"));
trace('// re.lastIndex;');
trace(re.lastIndex);
