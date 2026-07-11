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

var values = ["abc", true, 1.5, uint(2), -2, null, undefined, new Object(), ""];
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
        trace(e.getStackTrace());
    }

    try {
        trace("// Boolean.prototype.toString.call(value)");
        trace(Boolean.prototype.toString.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// Number.prototype.toString.call(value)");
        trace(Number.prototype.toString.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// int.prototype.toString.call(value)");
        trace(int.prototype.toString.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// uint.prototype.toString.call(value)");
        trace(uint.prototype.toString.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }
}
