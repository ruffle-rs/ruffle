// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {}
    }
}

import flash.events.EventDispatcher;
import flash.events.Event;

var f = function(){
    trace(this);
    try {
       trace(this.a);
    } catch(e) {}

    try {
        trace(this.parseFloat);
    } catch(e){}

    trace();
}
var t = this;
t.a = 123;

trace("// f()");
f();
trace("// f.call(null)");
f.call(null);
trace("// f.call(5)");
f.call(5);
trace("// f.apply(null, [])");
f.apply(null, []);


trace("// str.replace(pattern, f)")
var str:String = "abc12 def34";
var pattern:RegExp = /([a-z]+)([0-9]+)/;
str.replace(pattern, f);

trace("// event dispatch on `f`")
class C extends EventDispatcher {}
var c = new C();
c.addEventListener("x", f);
c.dispatchEvent(new Event("x"));


f = function(item, index, array) {
    trace(item);
    trace(this);
    try {
        trace(this.a);
    } catch(e) {}
    trace();
}

var arr = [1];
trace("// arr.forEach(f)")
arr.forEach(f);
trace("// arr.forEach(f, null)")
arr.forEach(f, null);
trace("// arr.forEach(f, undefined)")
arr.forEach(f, undefined);
trace("// arr.forEach(f, 5)")
arr.forEach(f, 5);
trace("// arr.forEach(f, {})")
arr.forEach(f, {});
