package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() {
            var a:Array = new Array();
            a[1] = "small";
            a[100] = "medium";
            a[2500000000] = "large";
            a[4000000000] = "huge";
            a[5000000000] = "beyond_u32";

            trace("length=" + a.length);

            var keys:Array = [];
            for (var k:String in a) {
                keys.push(k);
            }
            keys.sort(Array.NUMERIC);
            trace("keyCount=" + keys.length);
            for each (var key:String in keys) {
                trace(key + "=" + a[key]);
            }

            var values:Array = [];
            for each (var v:String in a) {
                values.push(v);
            }
            values.sort();
            trace("valueCount=" + values.length);
            for each (var vv:String in values) {
                trace("value=" + vv);
            }
        }
    }
}
