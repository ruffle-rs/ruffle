package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() {
        }
    }
}

var regexp = /abc/xsmig;

trace("// trace(regexp)");
trace(regexp);

trace("// regexp.toString()");
trace(regexp.toString());

trace("// RegExp.prototype.toString.call(regexp)");
trace(RegExp.prototype.toString.call(regexp));

trace("// Object.prototype.toString.call(regexp)");
trace(Object.prototype.toString.call(regexp));

trace("// RegExp.prototype.toString.call({})");
try {
    RegExp.prototype.toString.call({});
} catch (e) {
    trace(e);
}