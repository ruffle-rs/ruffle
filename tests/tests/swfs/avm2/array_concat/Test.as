// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

function assert_array(a: Array) {
	for (var i = 0; i < a.length; i += 1) {
		trace(a[i]);
	}
}

trace("//var a = new Array(\"a\", \"b\", \"c\");");
var a = new Array("a", "b", "c");

trace("//var b = new Array(\"d\", \"e\", \"f\");");
var b = new Array("d", "e", "f");

trace("//a.concat(b)");
var c = a.concat(b);

assert_array(c);

trace("//a.concat(\"d\", \"e\", \"f\");");
var d = a.concat("d", "e", "f");

assert_array(d);

trace("//a.concat(\"g\", b, \"h\");");
var e = a.concat("g", b, "h");

assert_array(e);

trace("//a.concat(b, b);");
var f = a.concat(b, b);

assert_array(f);

trace("//a.concat(null, undefined);");
var g = a.concat(null, undefined);

assert_array(g);
