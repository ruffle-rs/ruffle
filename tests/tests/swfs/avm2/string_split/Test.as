package {
	import flash.display.MovieClip;

	public class Test extends MovieClip {
		public function Test() {

// note: compiled manually with AIR SDK

trace('// var text = "a.b.c";');
var text = "a.b.c";

trace('// text.split("a.b.c")');
trace(text.split("a.b.c"));
trace('// text.split(".")');
trace(text.split("."));
trace('// text.split("")');
trace(text.split(""));

trace('// text.split()');
trace(text.split());

trace('// text.split(regex)');
var regex = /b+/
trace("abbabc".split(regex));

trace('// no match')
trace("ccccc".split(/b/));

trace('// match all')
var regex = /.*/
trace("cccc".split(regex));

trace('// empty string, match all')
trace("".split(/.*/));

trace('// multibyte chars')
trace("ąąbąą".split(/b/))

trace('// Group expansion')
trace("abba".split(/(b(b))/))

trace('// Split on empty regex')
trace("aął".split(/(?:)/))

trace('// Split on non-empty regex with zero-length match')
trace("aąbcde".split(/f*/))

trace('// Limit')
trace("aąbaababa".split(/b/,3))

trace('// Limit on group captures - flash returns 6 parts instead of 5')
trace("aąbbaabbabbabbabbabba".split(/(b(b))/,5))


		}
	}
}

