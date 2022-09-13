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
  trace(foobar.toString());
  trace(foobar.name);
  trace(foobar.errorID);
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
     trace(foobar.toString());
	 trace(foobar.name);
	 trace(foobar.errorID);
   }
}

trace("// Errors propagate through the stack");
try_passing();

}}}


