package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() {
        }
    }
}

trace("// String.prototype.toString()")
trace(String.prototype.toString());
trace("// Boolean.prototype.toString()")
trace(Boolean.prototype.toString());
trace("// Number.prototype.toString()")
trace(Number.prototype.toString());

var values = ["abc", true, 1.5, null, undefined];
for each (var value in values) {
    trace("");
    trace("// value");
    trace(value);

    trace("// Object.prototype.toString.call(value)");
    trace(Object.prototype.toString.call(value));

    try {
        trace("// String.prototype.toString.call(value)")
        trace(String.prototype.toString.call(value))
    } catch (e) {
        trace(e);
    }

    try {
        trace("// Boolean.prototype.toString.call(value)");
        trace(Boolean.prototype.toString.call(value))
    } catch (e) {
        trace(e);
    }

    try {
        trace("// Number.prototype.toString.call(value)");
        trace(Number.prototype.toString.call(value))
    } catch (e) {
        trace(e);
    }
}
