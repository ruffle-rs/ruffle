package {
	public class Test {
	}
}

trace('// new RegExp("").exec("");');
trace(new RegExp("").exec(""));

trace('// /\d+/.exec("abc");');
trace(/\d+/.exec("abc"));

trace('// /\d+/.exec("abc123");');
trace(/\d+/.exec("abc123"));

trace('// /ABC/i.exec("abc");');
trace(/ABC/i.exec("abc"));

trace('// /.bar/s.exec("foo\\nbar");');
trace(/.bar/s.exec("foo\nbar"));

// Test global and lastIndex
var re:RegExp = /(\w*)sh(\w*)/ig;
var result = re.exec("She sells seashells by the seashore");
trace(result);
trace("input", result.input);
trace("index", result.index);
trace("lastIndex", re.lastIndex);
result = re.exec("She sells seashells by the seashore");
trace(result);
trace("input", result.input);
trace("index", result.index);
trace("lastIndex", re.lastIndex);
