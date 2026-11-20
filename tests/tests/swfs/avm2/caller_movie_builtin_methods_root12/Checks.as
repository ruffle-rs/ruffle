package {
    // Methods whose behavior depends on the SWF version of the calling movie,
    // called directly, through the public prototype methods, and through the
    // public Function.prototype.call and Function.prototype.apply.
    public class Checks {
        private static var pub:Namespace = new Namespace("");

        public static function run():void {
            trace("// localeCompare");
            for each (var s:String in ["", "a"]) {
                for each (var other:* in [null, undefined]) {
                    var label:String = "\"" + s + "\".localeCompare(" + other + ")";
                    trace(label + ", direct: " + s.localeCompare(other));
                    trace(label + ", prototype: " + s.pub::localeCompare(other));
                    trace(label + ", call: " + s.localeCompare.pub::call(null, other));
                    trace(label + ", apply: " + s.localeCompare.pub::apply(null, [other]));
                }
            }

            trace("// sort NUMERIC");
            checkSort([268435455, -268435455, 1], Array.NUMERIC);

            trace("// sort with a comparator returning fractions");
            checkSort([3, 1, 2], function(a:*, b:*):Number {
                return (a - b) / 10;
            });
        }

        private static function checkSort(values:Array, arg:*):void {
            var a:Array;
            a = values.concat();
            trace("direct: " + a.sort(arg));
            a = values.concat();
            trace("prototype: " + a.pub::sort(arg));
            a = values.concat();
            trace("call: " + a.sort.pub::call(null, arg));
            a = values.concat();
            trace("apply: " + a.sort.pub::apply(null, [arg]));
        }
    }
}
