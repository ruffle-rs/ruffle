// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
	trace("// Trying to raise and catch an error")
try {
  var v = new Vector.<String>();
  v.fixed = true;
  v.push("a");
  v.push("bcd");
} catch(foobar) {
  // This cuts the error description to "RangeError"
  // which is a cheat to mask the fact that the full error
  // descriptions are different between Ruffle and Flash
  trace(foobar.toString().slice(0,10));
}

        }
    }
}


