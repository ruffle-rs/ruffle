package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() {
        }
    }
}

trace("// String.prototype.valueOf()")
trace(String.prototype.valueOf());
trace("// Boolean.prototype.valueOf()")
trace(Boolean.prototype.valueOf());
trace("// Number.prototype.valueOf()")
trace(Number.prototype.valueOf());

var values = ["abc", true, 1.5, uint(2), -2, null, undefined, new Object(), ""];
for each (var value in values) {
    trace("");
    trace("// value");
    trace(value);

    trace("// Object.prototype.valueOf.call(value)");
    trace(Object.prototype.valueOf.call(value));

    try {
        trace("// String.prototype.valueOf.call(value)")
        trace(String.prototype.valueOf.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// Boolean.prototype.valueOf.call(value)");
        trace(Boolean.prototype.valueOf.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// Number.prototype.valueOf.call(value)");
        trace(Number.prototype.valueOf.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// int.prototype.valueOf.call(value)");
        trace(int.prototype.valueOf.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }

    try {
        trace("// uint.prototype.valueOf.call(value)");
        trace(uint.prototype.valueOf.call(value))
    } catch (e) {
        trace(e.getStackTrace());
    }
}
