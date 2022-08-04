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

try {
  trace("// Check if scope is cleared on exception");
  var o = new Object();
  o["foo"] = 5
  with(o) {
    trace(typeof(foo));
    var v = new Vector.<String>();
    v.fixed = true;
    v.push("a");
  }
} catch(foobar) {
  var u = new Object();
  with (u) {  // wrapped in "with" to suppress compiler error on possibly undefined "foo"
    trace(typeof(foo));
  }
}

function triggerException() {
  var v = new Vector.<String>();
  v.fixed = true;
  v.push("a");  // throws error
}

function try_passing() {
   try {
      triggerException();
   } catch(foobar) {
     // Like above, cut the part where the error messages agree between Ruffle and FP
     trace(foobar.toString().slice(0,10));
   }
}

trace("// Errors propagate through the stack");
try_passing();

}}}


