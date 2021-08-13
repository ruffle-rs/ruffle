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

trace('// text.split() - unimplemented');
trace('// text.split(regex) - unimplemented');


		}
	}
}

