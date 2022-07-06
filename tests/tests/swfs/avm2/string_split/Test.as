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

// no match
trace("ccccc".split(regex));

var regex = /.*/
trace("cccc".split(regex));
trace("".split(regex));


		}
	}
}

